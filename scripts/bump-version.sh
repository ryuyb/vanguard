#!/usr/bin/env bash

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# 显示使用说明
show_usage() {
    cat << EOF
用法: $0 [选项] [版本号]

选项:
  -h, --help          显示此帮助信息
  -m, --major         升级主版本号 (x.0.0)
  -n, --minor         升级次版本号 (0.x.0)
  -p, --patch         升级修订号 (0.0.x) [默认]
  -s, --set VERSION   设置为指定版本号
  -r, --rc            创建或升级 RC 版本
  --release           从 RC 版本发布为正式版本
  -d, --dry-run       仅显示将要修改的内容，不实际修改
  -c, --commit        自动创建 git commit

示例:
  $0 -p              # 升级修订号: 0.1.0 -> 0.1.1
  $0 -n              # 升级次版本号: 0.1.0 -> 0.2.0
  $0 -m              # 升级主版本号: 0.1.0 -> 1.0.0
  $0 -s 1.2.3        # 设置为指定版本: 1.2.3
  $0 -n -r           # 创建 RC 版本: 0.1.0 -> 0.2.0-rc.1
  $0 -r              # 升级 RC 版本: 0.2.0-rc.1 -> 0.2.0-rc.2
  $0 --release       # 发布正式版: 0.2.0-rc.2 -> 0.2.0
  $0 -p -c           # 升级修订号并自动提交
  $0 -d -n           # 预览升级次版本号的效果
EOF
}

# 获取当前版本号
get_current_version() {
    local version=$(grep -m 1 '"version"' package.json | sed 's/.*"version": "\(.*\)".*/\1/')
    echo "$version"
}

# 解析版本号
parse_version() {
    local version=$1
    local major minor patch prerelease

    # 检查是否包含预发布版本号 (例如: 1.2.3-rc.1)
    if [[ $version =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)-(.+)$ ]]; then
        major="${BASH_REMATCH[1]}"
        minor="${BASH_REMATCH[2]}"
        patch="${BASH_REMATCH[3]}"
        prerelease="${BASH_REMATCH[4]}"
        echo "$major $minor $patch $prerelease"
    elif [[ $version =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
        major="${BASH_REMATCH[1]}"
        minor="${BASH_REMATCH[2]}"
        patch="${BASH_REMATCH[3]}"
        echo "$major $minor $patch"
    else
        echo ""
    fi
}

# 升级版本号
bump_version() {
    local current=$1
    local bump_type=$2
    local create_rc=$3

    local version_parts=($(parse_version "$current"))
    if [[ ${#version_parts[@]} -eq 0 ]]; then
        print_error "无法解析版本号: $current"
        exit 1
    fi

    local major="${version_parts[0]}"
    local minor="${version_parts[1]}"
    local patch="${version_parts[2]}"
    local prerelease="${version_parts[3]:-}"

    # 如果当前是 RC 版本
    if [[ -n "$prerelease" ]]; then
        if [[ "$create_rc" == "true" ]]; then
            # 升级 RC 版本号
            if [[ $prerelease =~ ^rc\.([0-9]+)$ ]]; then
                local rc_num="${BASH_REMATCH[1]}"
                rc_num=$((rc_num + 1))
                echo "$major.$minor.$patch-rc.$rc_num"
            else
                print_error "无法解析 RC 版本号: $prerelease"
                exit 1
            fi
        else
            # 从 RC 发布为正式版本
            echo "$major.$minor.$patch"
        fi
        return
    fi

    # 正常版本升级
    case $bump_type in
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        patch)
            patch=$((patch + 1))
            ;;
    esac

    # 如果需要创建 RC 版本
    if [[ "$create_rc" == "true" ]]; then
        echo "$major.$minor.$patch-rc.1"
    else
        echo "$major.$minor.$patch"
    fi
}

# 验证版本号格式
validate_version() {
    local version=$1
    # 支持标准版本号和 RC 版本号
    if [[ ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?$ ]]; then
        print_error "无效的版本号格式: $version"
        print_info "版本号格式应为: 主版本号.次版本号.修订号[-rc.RC号] (例如: 1.2.3 或 1.2.3-rc.1)"
        exit 1
    fi
}

# 更新文件中的版本号
update_file() {
    local file=$1
    local old_version=$2
    local new_version=$3
    local dry_run=$4

    if [[ ! -f "$file" ]]; then
        print_error "文件不存在: $file"
        exit 1
    fi

    if [[ "$dry_run" == "true" ]]; then
        print_info "将更新 $file"
        return
    fi

    case "$file" in
        package.json|src-tauri/tauri.conf.json)
            if [[ "$OSTYPE" == "darwin"* ]]; then
                sed -i '' "s/\"version\": \"$old_version\"/\"version\": \"$new_version\"/g" "$file"
            else
                sed -i "s/\"version\": \"$old_version\"/\"version\": \"$new_version\"/g" "$file"
            fi
            ;;
        src-tauri/Cargo.toml)
            if [[ "$OSTYPE" == "darwin"* ]]; then
                sed -i '' "s/version = \"$old_version\"/version = \"$new_version\"/g" "$file"
            else
                sed -i "s/version = \"$old_version\"/version = \"$new_version\"/g" "$file"
            fi
            ;;
    esac

    print_success "已更新 $file"
}

# 主函数
main() {
    local bump_type="patch"
    local custom_version=""
    local dry_run=false
    local auto_commit=false
    local create_rc=false
    local release_from_rc=false

    # 解析命令行参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_usage
                exit 0
                ;;
            -m|--major)
                bump_type="major"
                shift
                ;;
            -n|--minor)
                bump_type="minor"
                shift
                ;;
            -p|--patch)
                bump_type="patch"
                shift
                ;;
            -s|--set)
                custom_version="$2"
                shift 2
                ;;
            -r|--rc)
                create_rc=true
                shift
                ;;
            --release)
                release_from_rc=true
                shift
                ;;
            -d|--dry-run)
                dry_run=true
                shift
                ;;
            -c|--commit)
                auto_commit=true
                shift
                ;;
            *)
                print_error "未知选项: $1"
                show_usage
                exit 1
                ;;
        esac
    done

    # 检查是否在项目根目录
    if [[ ! -f "package.json" ]] || [[ ! -f "src-tauri/Cargo.toml" ]]; then
        print_error "请在项目根目录运行此脚本"
        exit 1
    fi

    # 获取当前版本
    local current_version=$(get_current_version)
    print_info "当前版本: $current_version"

    # 检查当前是否为 RC 版本
    local is_current_rc=false
    if [[ $current_version =~ -rc\. ]]; then
        is_current_rc=true
    fi

    # 计算新版本
    local new_version
    if [[ -n "$custom_version" ]]; then
        new_version="$custom_version"
        validate_version "$new_version"
        print_info "设置版本为: $new_version"
    elif [[ "$release_from_rc" == "true" ]]; then
        if [[ "$is_current_rc" == "false" ]]; then
            print_error "当前版本不是 RC 版本，无法执行 --release"
            exit 1
        fi
        new_version=$(bump_version "$current_version" "$bump_type" "false")
        print_info "从 RC 发布为正式版本"
        print_info "新版本: $new_version"
    elif [[ "$create_rc" == "true" ]]; then
        if [[ "$is_current_rc" == "true" ]]; then
            # 当前已是 RC，升级 RC 版本号
            new_version=$(bump_version "$current_version" "$bump_type" "true")
            print_info "升级 RC 版本号"
        else
            # 创建新的 RC 版本
            new_version=$(bump_version "$current_version" "$bump_type" "true")
            print_info "创建 RC 版本"
            print_info "升级类型: $bump_type"
        fi
        print_info "新版本: $new_version"
    else
        if [[ "$is_current_rc" == "true" ]]; then
            print_warning "当前是 RC 版本: $current_version"
            print_info "提示: 使用 --release 发布为正式版本，或使用 -r 升级 RC 版本号"
            exit 1
        fi
        new_version=$(bump_version "$current_version" "$bump_type" "false")
        print_info "升级类型: $bump_type"
        print_info "新版本: $new_version"
    fi

    # 检查版本是否相同
    if [[ "$current_version" == "$new_version" ]]; then
        print_warning "版本号未改变: $new_version"
        exit 0
    fi

    # 预览模式
    if [[ "$dry_run" == "true" ]]; then
        print_warning "预览模式 (不会实际修改文件)"
        echo ""
        print_info "将执行以下操作:"
        echo "  • package.json: $current_version → $new_version"
        echo "  • src-tauri/Cargo.toml: $current_version → $new_version"
        echo "  • src-tauri/tauri.conf.json: $current_version → $new_version"
        exit 0
    fi

    # 确认操作
    echo ""
    print_warning "即将更新版本号: $current_version → $new_version"
    read -p "确认继续? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "操作已取消"
        exit 0
    fi

    # 更新文件
    echo ""
    print_info "正在更新文件..."
    update_file "package.json" "$current_version" "$new_version" "$dry_run"
    update_file "src-tauri/Cargo.toml" "$current_version" "$new_version" "$dry_run"
    update_file "src-tauri/tauri.conf.json" "$current_version" "$new_version" "$dry_run"

    echo ""
    print_success "版本号已更新: $current_version → $new_version"

    # 自动提交
    if [[ "$auto_commit" == "true" ]]; then
        echo ""
        print_info "正在创建 git commit..."

        if ! git diff --quiet package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json; then
            git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json

            # 根据版本类型生成不同的 commit 消息
            local commit_msg
            if [[ $new_version =~ -rc\. ]]; then
                commit_msg="chore: bump version to $new_version (release candidate)"
            else
                commit_msg="chore: bump version to $new_version"
            fi

            git commit -m "$commit_msg"
            print_success "已创建 commit"

            # 创建 git tag
            read -p "是否创建 git tag v$new_version? [y/N] " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                local tag_msg
                if [[ $new_version =~ -rc\. ]]; then
                    tag_msg="Release Candidate v$new_version"
                else
                    tag_msg="Release v$new_version"
                fi

                git tag -a "v$new_version" -m "$tag_msg"
                print_success "已创建 tag: v$new_version"
                print_info "使用 'git push --tags' 推送标签到远程仓库"
            fi
        else
            print_warning "没有检测到文件变更"
        fi
    else
        echo ""
        print_info "下一步操作:"
        echo "  git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json"

        if [[ $new_version =~ -rc\. ]]; then
            echo "  git commit -m \"chore: bump version to $new_version (release candidate)\""
        else
            echo "  git commit -m \"chore: bump version to $new_version\""
        fi

        echo "  git tag -a \"v$new_version\" -m \"Release v$new_version\""
        echo "  git push --tags"
    fi
}

main "$@"
