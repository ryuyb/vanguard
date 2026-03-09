use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use rmpv::decode::read_value;
use rmpv::Value as MsgpackValue;
use serde_json::Value as JsonValue;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::task::JoinHandle;
use tokio::time::{timeout, Duration};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::AUTHORIZATION;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use crate::application::dto::notification::{NotificationConnectCommand, NotificationEvent};
use crate::application::ports::notification_port::NotificationPort;
use crate::support::error::AppError;
use crate::support::redaction::redact_sensitive;
use crate::support::result::AppResult;

const RECORD_SEPARATOR: char = '\u{001e}';
const INITIAL_HANDSHAKE_PAYLOAD: &str = r#"{"protocol":"messagepack","version":1}"#;

struct ActiveConnection {
    events_rx: Arc<Mutex<mpsc::Receiver<NotificationEvent>>>,
    stop_tx: Option<oneshot::Sender<()>>,
    reader_task: Option<JoinHandle<()>>,
    shutdown_timeout_ms: u64,
}

#[derive(Clone, Default)]
pub struct VaultwardenNotificationPort {
    connections: Arc<Mutex<HashMap<String, ActiveConnection>>>,
}

impl VaultwardenNotificationPort {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl NotificationPort for VaultwardenNotificationPort {
    async fn connect(&self, command: NotificationConnectCommand) -> AppResult<()> {
        require_non_empty(&command.account_id, "account_id")?;
        require_non_empty(&command.base_url, "base_url")?;
        require_non_empty(&command.access_token, "access_token")?;
        if command.queue_limit == 0 {
            return Err(AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "queue_limit must be greater than zero".to_string(),
            });
        }
        if command.connect_timeout_ms == 0 {
            return Err(AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "connect_timeout_ms must be greater than zero".to_string(),
            });
        }
        if command.handshake_timeout_ms == 0 {
            return Err(AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "handshake_timeout_ms must be greater than zero".to_string(),
            });
        }
        if command.shutdown_timeout_ms == 0 {
            return Err(AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "shutdown_timeout_ms must be greater than zero".to_string(),
            });
        }

        self.disconnect(&command.account_id).await?;

        let endpoint = websocket_endpoint(&command.base_url)?;
        let mut request =
            endpoint
                .into_client_request()
                .map_err(|error| AppError::NetworkRemoteError {
                    status: 0,
                    message: format!("failed to build websocket request: {error}"),
                })?;
        let bearer = format!("Bearer {}", command.access_token);
        let header_value =
            HeaderValue::from_str(&bearer).map_err(|error| AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: format!("invalid access token header: {error}"),
            })?;
        request.headers_mut().insert(AUTHORIZATION, header_value);

        let (mut stream, _) = timeout(
            Duration::from_millis(command.connect_timeout_ms),
            connect_async(request),
        )
        .await
        .map_err(|_| AppError::NetworkRemoteError {
            status: 0,
            message: format!(
                "websocket connect timeout after {}ms",
                command.connect_timeout_ms
            ),
        })?
        .map_err(|error| AppError::NetworkRemoteError {
            status: 0,
            message: format!("failed to connect websocket: {error}"),
        })?;

        let handshake_message = format!("{INITIAL_HANDSHAKE_PAYLOAD}{RECORD_SEPARATOR}");
        stream
            .send(Message::Text(handshake_message.into()))
            .await
            .map_err(|error| AppError::NetworkRemoteError {
                status: 0,
                message: format!("failed to send websocket handshake: {error}"),
            })?;

        if let Err(error) = wait_handshake_ack(&mut stream, command.handshake_timeout_ms).await {
            log::warn!(
                target: "vanguard::sync::ws",
                "websocket handshake failed account_id={} message={}",
                command.account_id,
                error
            );
            return Err(error);
        }
        log::info!(
            target: "vanguard::sync::ws",
            "websocket handshake succeeded account_id={}",
            command.account_id
        );

        let (events_tx, events_rx) = mpsc::channel::<NotificationEvent>(command.queue_limit);
        let (stop_tx, stop_rx) = oneshot::channel::<()>();
        let account_id = command.account_id.clone();
        let reader_task = tokio::spawn(async move {
            read_loop(account_id, stream, events_tx, stop_rx).await;
        });

        let mut connections = self.connections.lock().await;
        connections.insert(
            command.account_id,
            ActiveConnection {
                events_rx: Arc::new(Mutex::new(events_rx)),
                stop_tx: Some(stop_tx),
                reader_task: Some(reader_task),
                shutdown_timeout_ms: command.shutdown_timeout_ms,
            },
        );
        Ok(())
    }

    async fn next_event(&self, account_id: &str) -> AppResult<Option<NotificationEvent>> {
        require_non_empty(account_id, "account_id")?;

        let events_rx = {
            let connections = self.connections.lock().await;
            connections
                .get(account_id)
                .map(|connection| Arc::clone(&connection.events_rx))
        };

        let Some(events_rx) = events_rx else {
            return Ok(None);
        };

        let mut receiver = events_rx.lock().await;
        Ok(receiver.recv().await)
    }

    async fn disconnect(&self, account_id: &str) -> AppResult<()> {
        require_non_empty(account_id, "account_id")?;

        let mut active_connection = {
            let mut connections = self.connections.lock().await;
            connections.remove(account_id)
        };

        if let Some(connection) = active_connection.as_mut() {
            if let Some(stop_tx) = connection.stop_tx.take() {
                let _ = stop_tx.send(());
            }
            if let Some(reader_task) = connection.reader_task.take() {
                if timeout(
                    Duration::from_millis(connection.shutdown_timeout_ms),
                    reader_task,
                )
                .await
                .is_err()
                {
                    log::debug!(
                        target: "vanguard::sync::ws",
                        "websocket reader task shutdown timeout account_id={} timeout_ms={}",
                        account_id,
                        connection.shutdown_timeout_ms
                    );
                }
            }
        }

        Ok(())
    }
}

async fn wait_handshake_ack(
    stream: &mut WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    handshake_timeout_ms: u64,
) -> AppResult<()> {
    let deadline = tokio::time::Instant::now() + Duration::from_millis(handshake_timeout_ms);

    loop {
        let Some(remaining) = deadline.checked_duration_since(tokio::time::Instant::now()) else {
            return Err(AppError::NetworkRemoteError {
                status: 0,
                message: format!(
                    "websocket handshake ack timeout after {}ms",
                    handshake_timeout_ms
                ),
            });
        };

        let message =
            timeout(remaining, stream.next())
                .await
                .map_err(|_| AppError::NetworkRemoteError {
                    status: 0,
                    message: format!(
                        "websocket handshake ack timeout after {}ms",
                        handshake_timeout_ms
                    ),
                })?;

        match message {
            Some(Ok(Message::Text(text))) => {
                if is_handshake_ack_text(&text) {
                    return Ok(());
                }

                return Err(AppError::NetworkRemoteError {
                    status: 0,
                    message: format!("unexpected websocket handshake ack text payload: {text}"),
                });
            }
            Some(Ok(Message::Binary(payload))) => {
                if is_handshake_ack_binary(&payload) {
                    return Ok(());
                }

                return Err(AppError::NetworkRemoteError {
                    status: 0,
                    message: "unexpected websocket handshake ack binary payload".to_string(),
                });
            }
            Some(Ok(Message::Ping(payload))) => {
                stream.send(Message::Pong(payload)).await.map_err(|error| {
                    AppError::NetworkRemoteError {
                        status: 0,
                        message: format!("failed to send websocket pong during handshake: {error}"),
                    }
                })?;
            }
            Some(Ok(Message::Pong(_))) => {}
            Some(Ok(Message::Close(_))) => {
                return Err(AppError::NetworkRemoteError {
                    status: 0,
                    message: "websocket closed before handshake ack".to_string(),
                });
            }
            Some(Ok(other)) => {
                return Err(AppError::NetworkRemoteError {
                    status: 0,
                    message: format!("unexpected websocket handshake ack frame: {other:?}"),
                });
            }
            Some(Err(error)) => {
                return Err(AppError::NetworkRemoteError {
                    status: 0,
                    message: format!("websocket handshake ack failed: {error}"),
                });
            }
            None => {
                return Err(AppError::NetworkRemoteError {
                    status: 0,
                    message: "websocket closed before handshake ack".to_string(),
                })
            }
        }
    }
}

async fn read_loop(
    account_id: String,
    mut stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    events_tx: mpsc::Sender<NotificationEvent>,
    mut stop_rx: oneshot::Receiver<()>,
) {
    loop {
        tokio::select! {
            _ = &mut stop_rx => {
                let _ = stream.close(None).await;
                break;
            }
            frame = stream.next() => {
                match frame {
                    Some(Ok(Message::Binary(payload))) => {
                        match parse_signalr_payload(&payload) {
                            Ok(values) => {
                                for value in values {
                                    if let Some(event) = to_notification_event(value) {
                                        if !enqueue_event(&events_tx, &account_id, event).await {
                                            return;
                                        }
                                    }
                                }
                            }
                            Err(error) => {
                                log::warn!(
                                    target: "vanguard::sync::ws",
                                    "failed to decode websocket message account_id={} message={}",
                                    account_id,
                                    redact_sensitive(&error.to_string())
                                );
                            }
                        }
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        if let Err(error) = stream.send(Message::Pong(payload)).await {
                            log::warn!(
                                target: "vanguard::sync::ws",
                                "failed to send websocket pong account_id={} message={}",
                                account_id,
                                redact_sensitive(&error.to_string())
                            );
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Text(text))) => {
                        if !is_handshake_ack_text(&text) {
                            log::debug!(
                                target: "vanguard::sync::ws",
                                "ignored websocket text frame account_id={} payload={}",
                                account_id,
                                redact_sensitive(&text)
                            );
                        }
                    }
                    Some(Ok(Message::Close(frame))) => {
                        log::info!(
                            target: "vanguard::sync::ws",
                            "websocket closed account_id={} frame={:?}",
                            account_id,
                            redact_sensitive(&format!("{frame:?}"))
                        );
                        break;
                    }
                    Some(Ok(_)) => {}
                    Some(Err(error)) => {
                        log::warn!(
                            target: "vanguard::sync::ws",
                            "websocket read failed account_id={} message={}",
                            account_id,
                            redact_sensitive(&error.to_string())
                        );
                        break;
                    }
                    None => break,
                }
            }
        }
    }
}

async fn enqueue_event(
    events_tx: &mpsc::Sender<NotificationEvent>,
    account_id: &str,
    event: NotificationEvent,
) -> bool {
    match events_tx.try_send(event) {
        Ok(()) => true,
        Err(TrySendError::Closed(_)) => false,
        Err(TrySendError::Full(event)) => {
            if event.event_type == 11 {
                return events_tx.send(event).await.is_ok();
            }

            log::warn!(
                target: "vanguard::sync::ws",
                "dropping websocket event due to full queue account_id={} type={}",
                account_id,
                event.event_type
            );
            true
        }
    }
}

fn websocket_endpoint(base_url: &str) -> AppResult<String> {
    let normalized = base_url.trim().trim_end_matches('/');
    let endpoint = format!("{normalized}/notifications/hub");
    if endpoint.starts_with("https://") {
        return Ok(endpoint.replacen("https://", "wss://", 1));
    }
    if endpoint.starts_with("http://") {
        return Ok(endpoint.replacen("http://", "ws://", 1));
    }

    Err(AppError::ValidationFieldError {
        field: "unknown".to_string(),
        message: "base_url must start with http:// or https://".to_string(),
    })
}

fn is_handshake_ack_text(value: &str) -> bool {
    value == "{}" || value == format!("{{}}{RECORD_SEPARATOR}")
}

fn is_handshake_ack_binary(value: &[u8]) -> bool {
    value == b"{}\x1e"
}

fn parse_signalr_payload(bytes: &[u8]) -> AppResult<Vec<MsgpackValue>> {
    let mut cursor = 0usize;
    let mut values = Vec::new();

    while cursor < bytes.len() {
        let (frame_length, consumed_bytes) =
            decode_varint(&bytes[cursor..]).ok_or_else(|| AppError::NetworkRemoteError {
                status: 0,
                message: "invalid signalr websocket frame length prefix".to_string(),
            })?;
        cursor = cursor.saturating_add(consumed_bytes);

        let frame_end = cursor.saturating_add(frame_length);
        if frame_end > bytes.len() {
            return Err(AppError::NetworkRemoteError {
                status: 0,
                message: "signalr websocket frame length exceeds payload".to_string(),
            });
        }

        let frame_bytes = &bytes[cursor..frame_end];
        let value = read_value(&mut Cursor::new(frame_bytes)).map_err(|error| {
            AppError::NetworkRemoteError {
                status: 0,
                message: format!("messagepack decode failed: {error}"),
            }
        })?;
        values.push(value);
        cursor = frame_end;
    }

    Ok(values)
}

fn decode_varint(bytes: &[u8]) -> Option<(usize, usize)> {
    let mut value = 0usize;
    let mut shift = 0usize;

    for (index, byte) in bytes.iter().enumerate() {
        value |= usize::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Some((value, index + 1));
        }
        shift += 7;
        if shift > 28 {
            return None;
        }
    }

    None
}

fn to_notification_event(value: MsgpackValue) -> Option<NotificationEvent> {
    let MsgpackValue::Array(items) = value else {
        return None;
    };
    let message_type = items.first().and_then(msgpack_to_i64).unwrap_or_default();

    if message_type == 6 {
        return None;
    }
    if message_type != 1 {
        return None;
    }

    let target = items.get(3).and_then(msgpack_to_string).unwrap_or_default();
    if target != "ReceiveMessage" {
        return None;
    }

    let first_argument = items
        .get(4)
        .and_then(|value| match value {
            MsgpackValue::Array(arguments) => arguments.first(),
            _ => None,
        })?
        .clone();
    let MsgpackValue::Map(entries) = first_argument else {
        return None;
    };

    let event_type = map_get(&entries, "Type")
        .and_then(msgpack_to_i64)
        .and_then(|value| i32::try_from(value).ok())?;
    let context_id = map_get(&entries, "ContextId").and_then(msgpack_to_string);
    let payload = map_get(&entries, "Payload").map(msgpack_to_json);

    Some(NotificationEvent {
        event_type,
        context_id,
        payload,
        received_at_ms: now_unix_ms().unwrap_or_default(),
    })
}

fn map_get<'a>(entries: &'a [(MsgpackValue, MsgpackValue)], key: &str) -> Option<&'a MsgpackValue> {
    entries.iter().find_map(|(entry_key, entry_value)| {
        if msgpack_to_string(entry_key).as_deref() == Some(key) {
            Some(entry_value)
        } else {
            None
        }
    })
}

fn msgpack_to_string(value: &MsgpackValue) -> Option<String> {
    match value {
        MsgpackValue::String(inner) => inner.as_str().map(String::from),
        MsgpackValue::Binary(inner) => String::from_utf8(inner.clone()).ok(),
        _ => None,
    }
}

fn msgpack_to_i64(value: &MsgpackValue) -> Option<i64> {
    match value {
        MsgpackValue::Integer(inner) => inner
            .as_i64()
            .or_else(|| inner.as_u64().and_then(|v| i64::try_from(v).ok())),
        _ => None,
    }
}

fn msgpack_to_json(value: &MsgpackValue) -> JsonValue {
    match value {
        MsgpackValue::Nil => JsonValue::Null,
        MsgpackValue::Boolean(inner) => JsonValue::Bool(*inner),
        MsgpackValue::Integer(inner) => {
            if let Some(value) = inner.as_i64() {
                JsonValue::Number(value.into())
            } else if let Some(value) = inner.as_u64() {
                JsonValue::Number(value.into())
            } else {
                JsonValue::Null
            }
        }
        MsgpackValue::F32(inner) => serde_json::Number::from_f64(f64::from(*inner))
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        MsgpackValue::F64(inner) => serde_json::Number::from_f64(*inner)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        MsgpackValue::String(inner) => inner
            .as_str()
            .map(|value| JsonValue::String(value.to_string()))
            .unwrap_or(JsonValue::Null),
        MsgpackValue::Binary(inner) => JsonValue::Array(
            inner
                .iter()
                .map(|value| JsonValue::Number(u64::from(*value).into()))
                .collect(),
        ),
        MsgpackValue::Array(inner) => JsonValue::Array(inner.iter().map(msgpack_to_json).collect()),
        MsgpackValue::Map(inner) => {
            let mut result = serde_json::Map::new();
            for (key, value) in inner {
                let key = msgpack_to_string(key).unwrap_or_else(|| format!("{key:?}"));
                result.insert(key, msgpack_to_json(value));
            }
            JsonValue::Object(result)
        }
        MsgpackValue::Ext(_, inner) => JsonValue::Array(
            inner
                .iter()
                .map(|value| JsonValue::Number(u64::from(*value).into()))
                .collect(),
        ),
    }
}

fn now_unix_ms() -> AppResult<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| AppError::InternalUnexpected {
            message: format!("system clock before unix epoch: {error}"),
        })?;
    Ok(duration.as_millis().min(i64::MAX as u128) as i64)
}

fn require_non_empty(value: &str, field: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: format!("{field} cannot be empty"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmpv::encode::write_value;

    fn wrap_signalr_frame(value: MsgpackValue) -> Vec<u8> {
        let mut body = Vec::new();
        write_value(&mut body, &value).expect("failed to encode msgpack value");

        let mut length_prefix = Vec::new();
        let mut remaining = body.len();
        loop {
            let mut chunk = (remaining & 0x7f) as u8;
            remaining >>= 7;
            if remaining > 0 {
                chunk |= 0x80;
            }
            length_prefix.push(chunk);
            if remaining == 0 {
                break;
            }
        }

        let mut result = Vec::with_capacity(length_prefix.len() + body.len());
        result.extend(length_prefix);
        result.extend(body);
        result
    }

    #[test]
    fn decode_signalr_payload_extracts_invocation_frame() {
        let payload = wrap_signalr_frame(MsgpackValue::Array(vec![
            1.into(),
            MsgpackValue::Map(vec![]),
            MsgpackValue::Nil,
            "ReceiveMessage".into(),
            MsgpackValue::Array(vec![MsgpackValue::Map(vec![
                ("ContextId".into(), "device-1".into()),
                ("Type".into(), 5.into()),
                (
                    "Payload".into(),
                    MsgpackValue::Map(vec![("UserId".into(), "user-1".into())]),
                ),
            ])]),
        ]));

        let decoded = parse_signalr_payload(&payload).expect("expected parse success");
        assert_eq!(decoded.len(), 1);

        let event = to_notification_event(decoded[0].clone()).expect("expected notification event");
        assert_eq!(event.event_type, 5);
        assert_eq!(event.context_id.as_deref(), Some("device-1"));
        assert!(event.payload.is_some());
    }

    #[test]
    fn to_notification_event_ignores_ping_message() {
        let ping = MsgpackValue::Array(vec![6.into()]);
        assert!(to_notification_event(ping).is_none());
    }
}
