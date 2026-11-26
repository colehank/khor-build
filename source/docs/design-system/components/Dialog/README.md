# Dialog 与任务底栏

任务先决定操作含义，屏幕再决定摆放。2026-11-10 用户确认采用下面的统一约定；它取代旧版“所有弹窗都没有关闭按钮”“手机提交在底部”“机器头像即时写、名字等完成”的规则。

## 类型与出口

| 类型 | 语义 | Desktop | Mobile |
| --- | --- | --- | --- |
| 草稿编辑、新建 | 取消丢弃草稿；保存／添加／连接提交；失败保留内容 | 右下取消＋唯一主操作 | 顶部左取消、右主操作、中央标题 |
| 即时管理 | 已生效的设置不伪装成可取消；关闭离开 | 标题旁关闭；任务动作在底部 | 顶部关闭；任务动作在右侧 |
| 多步骤 | 返回上一步；保存子项后回父页 | 返回＋保存按任务位置排列 | 顶部左返回、右保存 |
| 只读预览 | 关闭；打开文件／导出是独立工具 | 标题旁标准关闭 | 顶部标准关闭 |
| 危险决定 | 取消＋明确的删除／重置动作 | 公共短确认布局 | 公共短确认布局，不扩大成长表单 |
| 就近选择 | 选择后回原处；多选菜单支持连续选择 | PopupButton／就近菜单 | 同样的就近菜单 |
| 持续浏览 | 会话／文件／网页自己的返回、收起、关闭 | 侧区与标签栏 | 原有药丸与内容甲板 |

桌面右下提交是 **Khor 的应用约定**；Apple HIG 不要求所有 macOS 弹窗的按钮都在右下。`Back` 不是关闭整个任务，`Cancel` 不能承诺撤销已经落盘的操作。邀请凭据、认证与安装进度等已经产生副作用的流程用关闭。

## 公共组件

```tsx
<Dialog open={open} onOpenChange={setOpen}>
  <TaskOverlay purpose="edit">
    <DialogTitle>重命名</DialogTitle>
    <DialogBody>{/* 草稿内容 */}</DialogBody>
    <TaskActions dirty={name !== original} busy={saving}
      cancel={<Button onClick={onClose}>取消</Button>}
      primary={<Button onClick={save}>重命名</Button>}/>
  </TaskOverlay>
</Dialog>
```

- `TaskActions` 声明 `cancel`、`back`、`close` 之一，以及可选 `primary`、`tools`。可以放在持有状态的子表单里，但一个任务同时只有一组操作；不自行定位按钮、不读取按钮文字猜用途。
- `TaskOverlay` 负责标题与操作的固定布局；正文只有一个公共滚动区。`DialogBody` 在任务内只是内容分组；预览器自身的文档滚动仍由预览器负责。
- 同一组按钮跨断点移动位置，输入控件不重挂载。不要通过改 App 窗口大小获得空间。
- 一个任务一个主按钮；导出、复制、删除自定义配色等放 `tools`，不与提交混成一排同等强调操作。
- 手机顶部主操作共用 `Button compactTouch`：32 的可见胶囊、至少 44 的点击区域；桌面尺寸不变。附件选择面板采用同一呈现。
- `dirty` 只表示未提交草稿。Escape／遮罩／下拉关闭统一请求退出：有草稿先询问保留编辑或丢弃；显式取消／返回是明确放弃当前草稿。`busy` 表示不可取消的写入，操作期间不退出、不重复提交；可取消认证用关闭主动中止等待。
- 没有操作槽位的只读／选择任务提供标准关闭。非任务内容需要明确 `layout="content"`，目前仅会话标签总览、文件／提交历史甲板、网络图机器卡；它们已有完整导航。
- 长编辑用 `initialHeight="large" maxHeight="large"`；选择默认中档到大档；短编辑适配内容。只有能拖动／改变高度的任务才显示抓手。
- 标题、面、圆角、间距和控件高度用现有 tokens。桌面浮层 `popover`＋`float`＋`hairline`；手机任务底栏沿用 `background`、上沿线与 `menu-r`。遮罩统一 `sheet-scrim`。

文件与目录这种持续浏览页面使用 `ContentOverlay` 组合现有 `TaskOverlay`：保留公共遮罩、抓手、焦点和手势，内容不再继承表单的横向与底部内边距。里面直接放正式 `FilePage`／`FilesPane`，由页面负责自己的内容边距和导航安全区。附件预览与机器目录使用同一个容器，避免预览再次缩窄。手机顶部为抓手保留完整 `--touch` 高度，第一条可点击内容不能进入抓手的点击区；这不增加横向或底部留白。

`ContentDeck`／`ContentPill` 是现有会话内容底栏的公共行与药丸；工作区文件页也使用同一个药丸。头像放在 `PillFace` 的居中座位内，与 session 的 `DeckBand` 共用占位，不把头像本身直接贴到胶囊边缘。实际 App Lab 的附件流程检查预览完整内容宽度、头像座位、底栏对齐与终端尺寸保持。

## 边界与手势

所有弹窗以 `visualViewport` 的高度和偏移为边界，包括软键盘、安全区和低矮桌面窗口。长内容能滚到最后一项，主操作始终可达。菜单的手势属于菜单，不能拉动外层底栏；路径补全参与正文高度，不被裁掉。

`overlay-policy.ts` 决定呈现与停靠；`use-task-overlay` 处理内容高度；`use-sheet-pan` 处理拖动、滚动交接和取消。非输入类任务不自动唤起键盘。任务之间替换时，退出的任务不能抢走新任务焦点；真正关闭时回到打开控件。嵌套步骤使用同一容器或替换任务，不叠两张任务 sheet；放弃草稿是短确认。

## 验证

`npm run dialogs` 验证真实公共组件与表单的视口、操作位置、草稿退出、长列表、焦点和手势归属；`npm run dialogs -- --serve` 提供一次性数据的检查入口。真实 App Lab 同时覆盖键行、配色和设置，发布前另做 iOS 键盘／惯性／安全区与桌面窗口验收。合成 touch 探针不冒充 WKWebView 原生实测。

官方依据：[Sheets](https://developer.apple.com/design/human-interface-guidelines/sheets)、[Modality](https://developer.apple.com/design/human-interface-guidelines/modality)、[Popovers](https://developer.apple.com/design/human-interface-guidelines/popovers)、[Toolbars](https://developer.apple.com/design/human-interface-guidelines/toolbars)、[Alerts](https://developer.apple.com/design/human-interface-guidelines/alerts)。

### 附件的选择区与已选区（2026-11-17）

键盘位置的附件面板使用不透明的 `canvas-solid`，不随原生窗口透明度穿透；机器目录与文件预览的 `ContentOverlay` 同样持有不透明背景。顶部只保留取消、发送。正文按“选择文件 → 已选文件 → 发送到”排列，三个区块同级标题与边界对齐，组间、标题与控件之间使用现有间距。

附件多选圆圈的选中态使用与设置开关一致的 `brand-strong`，勾用 `on-brand` 保证浅深主题对比；未选中为空心圆。文件行背景仍遵循中性 `fill-strong`，不用绿色铺整行。已选列表与机器目录选择行共用 `FileChoiceRow`。

选择文件分两行：本机的文件／目录动作；其他机器的头像＋机器名。完整机器条目放不下时收进“…”选择菜单，不让半截条目冒充可点击项。本机不重复进入其他机器行。空选择隐藏整块已选文件；发送目录紧随已选区。短键盘内正文可滚动，控件保持正常单行与触摸尺寸；已选长列表仍独立滚动。

已选区复用 `AttachmentList` 与 `FileChoiceRow`，文件大小采用工作区同一格式并独立于可截断的来源说明。原生路径缺少大小时只从来源机读取事实，不提前传输；已知大小不重复读取，未知目录不伪造总大小。实际 App Lab 验证背景、分区、响应式机器菜单、来源机大小、空选择、取消／发送与终端网格保持。

桌面终端不提供顶部回形针按钮，直接拖放进入待发送草稿；手机仍由键行附件按钮进入。拖放不自动发送。

GUI 与 Khor 对话输入框的「＋」直接打开同一个附件选择面板，不再先弹添加菜单，也不保留查找／导出对话的菜单项。桌面直接拖入文件也进入该待发送面板，不提前传输；附件发送继续使用各自会话的发送逻辑。GUI 的文件限制由协议能力决定，不复制终端的传输实现。
