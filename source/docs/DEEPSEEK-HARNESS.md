# DeepSeek Harness

DeepSeek Harness is available as `dsh` in Khor's GUI conversations and in the Khor
agent's delegation tools. It uses the official ACP v1 profile:

```sh
dsh --profile acp
```

Install DSH and configure its model credentials on the **execution computer**.
Khor resolves that computer's `dsh` command and uses its DSH profile and credential
store. Registering another computer does not install DSH there or copy credentials.
The launch default is provided without writing a registration to the synced agent
document. An explicit `dsh` registration takes precedence, for example for a
different installation or an ACP-capable custom profile:

```sh
khor agents add dsh -- /absolute/path/to/dsh --profile my-acp
```

In New Session, choose the computer and workspace, then **DeepSeek Harness**.
The optional capability check uses the selected computer. Models, reasoning
strength, tools and permissions come from DSH's actual protocol responses.
Model routing values remain opaque; the GUI displays the advertised model names.
The shared session mark is the official Harness whale in both themes.

In a Khor conversation, ask it to delegate to `dsh` on the desired computer.
The `agents` tool reports whether the command is available. `delegate`,
`task_status`, `task_wait`, `task_answer` and `task_cancel` use the normal owned
ACP session and task receipts. Finding the command does not validate credentials.

The official ACP profile exposes committed messages and generic tool events,
model/configuration options, one-shot permissions, cancellation, listing and
resume. It does not provide DSH's complete Web UI, raw token streaming,
interactive questions, plans, slash commands, additional workspace directories,
or transcript replay. Image input is enabled only when DSH advertises it. Files
can still be delivered to the execution machine and referenced by path. Khor
records the events of its own conversations independently of provider replay.

DSH remains in developer preview. The local acceptance baseline is
`0.2.1-rc.5`; custom installations retain their selected version. Khor does not
silently download or upgrade DSH. Upstream protocol changes require renewed
compatibility checks.

The DSH Lab scene uses the complete App with isolated protocol data:
`/app/?scene=dsh&machines=3&items=5`. It does not make model calls. Native/ACP and
real-model acceptance evidence is recorded separately in
`docs/handoff/deepseek-harness-20261117.md`.
