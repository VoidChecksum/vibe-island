const vscode = require("vscode");

function activate(context) {
  const handler = vscode.window.registerUriHandler({
    handleUri(uri) {
      const params = new URLSearchParams(uri.query);
      const pid = params.get("pid");
      const tty = params.get("tty");
      const title = params.get("title");

      const terminals = vscode.window.terminals;
      let target = null;

      // Match by pid first
      if (pid) {
        for (const t of terminals) {
          if (t.processId && String(t.processId) === pid) {
            target = t;
            break;
          }
        }
      }

      // Fallback: match by tty path in terminal name
      if (!target && tty) {
        const ttyBase = tty.split("/").pop();
        for (const t of terminals) {
          if (t.name && t.name.includes(ttyBase)) {
            target = t;
            break;
          }
        }
      }

      // Fallback: match by title substring
      if (!target && title) {
        for (const t of terminals) {
          if (t.name && t.name.includes(title)) {
            target = t;
            break;
          }
        }
      }

      if (target) {
        target.show(false);
      }
    },
  });

  context.subscriptions.push(handler);
}

function deactivate() {}

module.exports = { activate, deactivate };
