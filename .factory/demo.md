# Demo sandbox

## One-click web demo

Open `/demo/` or select **Try it with sample data** on the landing page. The
page shows the bundled checkout-flyout output immediately. Its persistent
banner says **Demo — sample data, nothing is saved**. **Reset demo** reloads
the fixed sample. **Start for real** takes the visitor to the install steps.

The web page does not use browser storage, so it cannot read or modify a real
workflow. Its sample output is a view of the same bundled CLI workflow.

## CLI demo

After installing the binary, run:

```sh
zoomcheck demo
```

The command writes `sample/checkout-flyout.html`, `sample/workflow.json`,
`report.json`, screenshots, and `index.html` into a new `zoomcheck-demo-*`
temporary directory and prints that path. The sample intentionally contains a
clipped payment control, so the command exits `1` after producing its report.
Pass `--out path/to/report` to select a disposable output folder. No real
workflow path is read or written by this command.
