# 跨平台终端启动与结束会话

日期：2026-11-18。记录本轮用户确认的要求、现场事实与实现。用户已授权独立 worktree/分支实施并在验证通过后合并 main；候选实现已落地，正在远端验证。

## 用户确认的启动流程

Khor 需要支持 Windows、Linux、macOS。用户或 Khor agent 新建交互终端时，目标机器创建 PTY（Windows 使用 ConPTY），按该平台及用户选择启动 shell，加载用户环境并接入 Khor shell 集成；shell 和终端能够交互后，再把用户填写的启动命令作为终端输入发送一次并回车。

例如 `cd ~/project && codex --resume xxx` 应由这个交互 shell 解析，保留用户的别名、函数、环境变量、管道和重定向语义。重连、多端查看与创建重试均不能重复执行启动命令。用户已经纠正了此前把“用户启动命令”与底层直接执行 argv 接口混在一起的说法；后续实现不得据此把用户的启动命令改成直接启动 Codex 或 tmux。

shell、HOME、PATH、cwd 和用户身份由实际运行会话的目标机器解析。跨机器创建不能采用发起端的路径和 shell。

调查时的 `shell_start::StartupInput` 已缓存启动命令，并在宿主收到 `ClientOp::Ready` 后释放一次。Ready 表示终端回放及查询应答已就绪，不能单凭它断言用户 shell 的初始化已完成；本轮实现分别等待 shell 和终端就绪。背景见 [自动启动命令等待终端就绪](2026-10-22-terminal-startup.md)。

## MacBook Air 的 Homebrew 问题

实际会话为 `shell/7c3c9dee5dcb06ec`，列表名称 `zgh`，home 为 MacBook Air。其命令记录显示 `brew` 和 `khor` 找不到，执行 `source ~/.zshrc` 后 `brew` 仍找不到。

该机器的 `~/.zprofile` 第 7 行是 `eval "$(/opt/homebrew/bin/brew shellenv)"`。新终端的 `open_shell_here_from` 只传入 `[shell]`，没有登录参数；zsh 的交互非登录模式不读取 `.zprofile`。

`spawn_host_with` 已将探测的用户 PATH 通过 `KHOR_PATH` 传给宿主，但 `host_serving` 使用 `spawn_path(&cmd[0])`：只有原 PATH 无法找到启动程序时才采用 `Node::user_path()`。绝对路径 `/bin/zsh` 本身存在，故即使用户环境已探测，也可能沿用缺少 Homebrew 的原 PATH。这是 Khor 的启动逻辑缺口，手工加载配置只是临时绕过。

代码入口：`crates/node/src/shell_start.rs`、`link.rs::open_shell_here_from`、`host.rs::spawn_host_with` / `host_serving`、`lib.rs::spawn_path` / `Node::user_path`。

## 跨平台实现方向

以下为讨论时记录的方向，已实施范围及验收入口见末节；其中 WSL 的发行版选择不在本轮实现范围：

- macOS 默认交互终端采用登录 shell 语义，参考 iTerm2 和 Ghostty 对 `login`、工作目录和登录环境的处理。
- Linux 默认遵循用户会话环境与交互 shell 习惯，登录模式可配置；后台启动时需要明确用户环境来源，不能只继承不完整的服务环境。
- Windows 使用 ConPTY 与原生进程创建方式，支持用户选择的 PowerShell、cmd 等。PowerShell 正常加载 Profile；不能照搬 Unix 的 `-l`。
- WSL 需要明确发行版和用户，在 WSL 内解析 shell 与 Linux 路径。
- shell 集成按 shell 适配，失败时允许基础终端继续工作，不修改用户启动文件来弥补 Khor 的启动错误。
- 环境恢复独立于“启动程序是否找到”。探测需要超时、可靠的结果通道、缓存刷新和可见的失败原因；PATH 分隔、环境变量大小写、程序查找和引号规则遵循目标平台。
- 内部直接运行程序的接口需要保留 argv 语义，但这不改变上面的产品启动命令流程。

验证应覆盖桌面启动、后台常驻、CLI 启动、远端创建、自定义 shell、空格和 Unicode 路径、慢启动配置、失效 shell、环境变量更新，以及重连后命令只执行一次。

参考：[iTerm2 启动配置](https://iterm2.com/documentation-preferences-profiles-general.html)、[Ghostty 启动实现](https://github.com/ghostty-org/ghostty/blob/main/src/termio/Exec.zig)、[zsh 启动文件](https://zsh.sourceforge.io/Doc/Release/Files.html)、[Bash 启动文件](https://www.gnu.org/software/bash/manual/html_node/Bash-Startup-Files)、[PowerShell Profile](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_profiles)、[ConPTY](https://learn.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session)。

## 已结束但打开空白的会话

用户反馈：agent 或用户自己创建的 session 会进入 session list，经常显示已结束，点开无内容。相关实例已由用户删除，无法逐条确认其原始命令、退出码和日志；当前实际列表没有此类已结束项。

修复前确认的代码事实（以下描述旧版本）：

1. MCP `open` 经 `Node::open_on` 启动的是 `sh -c command`。这种一次性命令执行结束后，宿主记录子进程退出，session 随之结束；它没有先开一个等待下一条输入的交互 shell。该入口与用户确认的终端创建流程存在差异。
2. Khor agent 的 `run_command` 是另一条任务回执路径，保存在父任务下，不能把所有 agent 命令都视为调用了 MCP `open`。
3. 结束会话的输出原应由宿主持久化到 `ring.bin`，没有屏幕时回退到 `commands.json`。启动错误另存 `host-error.txt`，当前 `Ended` 数据并未包含这个错误文件。
4. `apps/gui/core/src/lib.rs` 只为本机死会话填充 `ended`；`term::ended` 也只读取本机。远端已结束会话的输出取回尚缺对应路由，不能用本机查找替代。
5. `TerminalPane.tsx::EndedPane` 将读取异常转换为 `null`，且加载中与 `null` 都渲染空白区域，没有给出失败原因。

因此，“为什么结束”和“结束后为什么空白”必须分别诊断。登录环境可能导致命令找不到或启动失败，但不能解释全部结束会话；一次性命令正常结束、宿主丢失、输出留存及远端读取也都需要覆盖。

## MCP 实测

实测使用已安装的 `/Applications/Khor.app/Contents/MacOS/khor mcp`，版本 `0.2.0`，通过真实 stdio JSON-RPC 执行 `initialize`、`tools/list` 和 `tools/call`。测试使用独立 KHOR_HOME 和未配对的测试身份；PATH 与 KHOR_PATH 均明确设为 `/usr/bin:/bin:/usr/sbin:/sbin`，隔离验证生命周期与环境两类问题。

| MCP open 的 command | 实测结果 | 输出留存 |
|---|---|---|
| `printf 'KHOR-MCP-SHORT-OK\n'` | exit 0，MCP session list 保留为 done | `ring.bin` 保存了标记文本，命令账本为空 |
| `true` | exit 0，列表保留为 done | 无 `ring.bin`，无命令记录 |
| `printf 'KHOR-MCP-ENV-FAIL\n'; brew --version` | exit 127，列表显示 failed | `ring.bin` 有输出；这是受控的 PATH 缺失案例 |
| `exec /bin/zsh -f` | exit 未设置，交互 shell 保持 idle | 没加载用户登录配置也仍然存活 |

另通过 MCP 创建 `exec /bin/zsh -f` 会话，再使用已安装 CLI 的真实 PTY attach 输入两条命令。第一条 `printf 'KHOR-FIRST-COMMAND\n'; false` 执行后，第二条 `printf 'KHOR-SECOND-COMMAND\n'` 仍成功输出；session exit 仍为空，attachment 仍活着。验证按独立输出行匹配，未把回显的输入当作执行结果。测试后经 MCP close 关闭交互会话，其余测试宿主均已退出。

这证明 MCP open 的一次性命令会话自然结束，且有无登录环境并不决定交互 shell 是否存活；正常结束但没有输出也可独立复现。GUI 空白分支和远端结束内容读取缺口由本轮源码检查确认，未将这次 MCP 验证描述为原生 App 页面复现，也不能据此追认用户已删除实例的具体原因。

证据目录：`/Volumes/disk0/Khor/evidence/20261118-session-startup-r9wc9a5q`。其中 `results.json` 为四组对照，`interactive-result.json` 为真实终端输入结果，`mcp-transcript.jsonl` / `interactive-mcp-transcript.json` 为 MCP 请求响应，`cleanup-check.json` 为宿主退出核验。未修改产品源码、合并或发布。

## MCP 能力与列表归属

用户确认 MCP 应同时提供一次性执行与常驻交互终端，一次性执行不留在 session list；无论由 MCP 还是 CLI 发起都应一致。智能体根据用户意图和任务需要选择；长时任务可以选择常驻，让用户查看，也让智能体持续监控。以下记录已确认要求，当前实现见末节：

- 一次性执行作为命令任务管理，返回退出码、stdout、stderr 和必要的任务标识。结果保留在发起对话、任务回执或命令记录中，不进入常驻 session list。运行较久时提供查询、输出读取和取消，不因运行时间长自动变成终端会话。
- 常驻交互终端作为 session 管理，进入 session list，遵循“先启动 shell 并加载环境，再输入启动命令”的流程。启动命令结束后返回提示符，session 继续存在；MCP 应配套提供读输出、发送输入和关闭能力。
- 归属由创建时的明确意图决定。智能体可依据预期运行时长、是否需要交互、用户是否需要持续查看、任务是否需要持续监控选择模式；用户明确指定时以用户意图为准。各入口调用同一套生命周期和列表规则，不按运行了多少秒自动切换模式。
- 一次性任务成功、失败、超时或取消都需要保留可读取的结果，再按留存规则回收资源；从 session list 移除不能等同于丢弃错误和输出。
- 真正的交互 session 异常退出时，保留退出原因及最后输出，供用户排查；其结束记录与一次性任务的列表归属分别处理。

## 用户与智能体共同查看常驻任务

常驻 session 需要同时支持用户界面和智能体工具访问同一个会话身份、输出及状态。用户能从列表进入终端，智能体能凭 session id 读取增量输出、等待状态变化，判断正在执行、已回到提示符、等待输入或已异常结束，并在任务授权范围内继续操作或结束任务。会话 shell 仍活着与启动命令仍在运行是两件事，监控必须能区分，不能仅用宿主 PID 存活判断任务进度。

智能体创建常驻会话时应提供有意义的标题，并在发起对话中留下该 session 的引用和用途，让用户能找到它。监控与查看不依赖用户是否打开终端页面，也不重复执行启动命令。增量读取和等待接口需要游标及可见的输出截断信息，避免反复读取全部历史或高频轮询。

输入控制使用宿主代次、输入版本和稳定请求标识协调；用户正在编辑一行时拒绝智能体写入。只读监控可以与用户查看同时进行，不改变终端尺寸。shell 配置中的交互询问允许用户即时回答，自动启动命令继续等到新的 shell 就绪信号后才释放。

## 实现分支与验收入口

本轮 worktree 为 `/Volumes/disk0/Khor/worktrees/current/khor-session-execution-20261118`，分支 `feat/session-execution-20261118`，基线 main `2b5d350a6b043aaf02346d9a5dd3f8c1c3109325`。实现包括原生 shell 启动与有界环境探测、独立命令任务、共用 IPC/配对机器路由、MCP/CLI 监控与输入、常驻会话代次及输入版本保护，以及本机/远端结束内容和读取状态。完整 API 与生命周期说明见 [SESSION](../SESSION.md) 和 [AGENT](../AGENT.md)。

外部 MCP 提供 `execute` 与 `open`；内部 Khor agent 使用归属父轮次的 `run_command` 与 `open`，避免独立任务绕过停止/超时。命令任务可查询、增量读取、取消和清理输出，清理后仍保留去重身份。CLI `run` 的普通命令不创建 session，显式 `--tui` 保留终端托管入口；默认 `open` 启动交互 shell，原样 `--command` 与结构化 argv 均由目标 shell 处理。

原生验证由 `scripts/session_execution_checks.py` 驱动，Quality 调用三平台 `session-execution.yml`；含真实 detached worker、真实 MCP 与 PTY、控制读取不改变终端尺寸、输入代次/版本/人类编辑保护、配对 IPC/QUIC 结束内容读取。GUI 门为 `npm run ended-terminal` 与 `CHAT_RECOVERY_SCENARIOS=retained-remote-ended-shell npm run chat-recovery`。本机仅做源码、格式和薄 PTY 探测，构建与正式测试通过既有远端入口执行。

执行状态与确切源码/日志持续记录在 `/Volumes/disk0/Khor/evidence/session-execution-20261118/STATE.md`。当前不能把源码审查与薄探测当成三平台/GUI 已通过；首轮远端已暴露并修正格式及类型接线错误，后续结果以对应快照和可信 Quality 收据为准。
