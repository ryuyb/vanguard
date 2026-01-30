import "./App.css";
import {invoke} from "@tauri-apps/api/core";
import {Button} from "@/components/ui/button.tsx";

function App() {
  const test = async () => {
    await invoke("login_and_sync", {serverUrl: "", email: "", password: ""});
  }

  return (
    <main>
      <Button onClick={test}>test</Button>
    </main>
  );
}

export default App;
