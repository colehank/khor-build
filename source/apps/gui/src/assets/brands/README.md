# Session identity artwork

`deepseek-harness.svg` is the official DeepSeek Harness whale from
[the upstream Web favicon](https://github.com/deepseek-ai/deepseek-harness/blob/39163dd30f703a79079e2d2446545cff3c453738/apps/web/public/favicon.svg).
The official `dsh-client-ui-brand-official` plugin also renders the whale as its
`OfficialBrandMark` through `FishLogo`.

Its path geometry is unchanged. Exporter IDs and redundant opacity were removed,
black became `currentColor`, and the view box was centered for the shared brand
live area. `BrandIcons.IconDsh` owns its use; `sessionMark` and `SessionFace` share
it across creation, rows, desktop tabs and mobile navigation. Shell, transfer and
borrow retain their own type marks. This asset identifies the connected agent;
it is not Khor's application icon or a claim of DeepSeek endorsement.
