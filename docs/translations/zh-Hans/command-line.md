---
source: 5b0744accf923762bf8a9e744e70cb250be2483b
---

# 命令行 { #command-line }

```sh
trenchdot [options] [map]
```

不带选项时编辑器直接打开，如果给了地图就带着这张地图打开。`trenchdot --help`（或 `-h`）会列出这些选项并退出。

| 选项 | 作用 |
| --- | --- |
| `--project <dir>` | 加载 `<dir>` 或它上层文件夹里的 Godot 项目，并在本次会话中信任它，参见[项目信任](godot/running-godot.md#project-trust)。不带它时编辑器会加载上一次的项目 |
| `--default-prefs` | 用默认偏好设置启动，并且从不保存它们。适合脚本和测试，也可以用来判断问题是不是出在你自己的设置上 |
| `--offscreen` | 把窗口打开在所有屏幕之外，没有焦点，也没有任务栏按钮。它仍然会渲染，所以脚本和测试可以截图，而你可以继续做自己的事 |
| `--live-link-port=<port>` | 仅在这一次运行中，用这个端口而不是偏好设置里的 *Godot Live Link* 端口去和 Godot 编辑器通信。脚本和测试会传一个空闲端口，这样它们的保存就不会重建你开着的 Godot 里的地图 |
| `--hot-reload-port=<port>` | 对 *运行中游戏热重载* 端口做同样的事，运行中的游戏会监听这个端口来重新加载已保存的地图 |
| `--dump <map>` | 把地图以 JSON 输出，然后退出 |
| `--to-json <map> <out>` | 把地图写成 JSON，然后退出 |
| `--to-tdm <map> <out>` | 把地图写成二进制 `.tdm`，然后退出 |
| `--export-glb <map> <out.glb>` | 把地图的几何体和材质写成 glTF 二进制，供 Blender 和其他 3D 工具使用，然后退出 |
| `--export-obj <map> <out.obj>` | 写出同样的内容，格式为 OBJ，旁边带一个 `.mtl` 和一个纹理文件夹，然后退出 |

遇到未知选项，或者缺少值的选项，会输出错误并以代码 2 退出，不会打开窗口。转换或导出失败时会输出原因并以代码 1 退出。

转换和导出从不打开窗口，所以可以放心地用在脚本和 git 钩子里，参见[git 里的地图文件](development.md#map-files-in-git)。

导出会从地图所在的文件夹找到它的 Godot 项目，并使用 *文件 > 导出* 的默认设置：整张地图，不含隐藏的对象和散布实例。实体逻辑、触发器和脚本都不会导出，参见[导出到 Blender 和其他 3D 工具](editor/exporting.md)。

<div class="td-mcp" markdown>

## MCP { #mcp }

这些选项用来设置 [MCP 服务器](mcp.md)，让脚本和 AI 代理可以驱动编辑器。

| 选项 | 作用 |
| --- | --- |
| `--mcp` | 通过 stdin 和 stdout 提供 MCP |
| `--mcp-http[=<port>]` | 通过 HTTP 在 127.0.0.1 上提供 MCP，默认端口 7841 |
| `--mcp-token=<token>` | 要求 HTTP 客户端发送这个令牌，不管偏好设置里写的是什么。它优先于 `TRENCHDOT_MCP_TOKEN`，参见 [HTTP 令牌](mcp.md#the-http-token) |

`--default-prefs` 和这两个选项都很搭，这样脚本化的会话每次都从同样的设置开始。设置了 `TRENCHDOT_DISABLE_MCP=1` 时，这两个选项都不会启动服务器，参见[关闭 MCP](mcp.md#turning-mcp-off)。

</div>
