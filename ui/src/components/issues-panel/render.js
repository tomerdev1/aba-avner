export function renderIssues({ title, hint, items }, elements) {
  if (!elements.issuesPanel || !elements.issuesList || !elements.issuesBadge || !elements.issuesHint) {
    return;
  }

  elements.issuesList.replaceChildren();

  if (!items || !items.length) {
    elements.issuesPanel.hidden = true;
    elements.issuesBadge.textContent = "";
    elements.issuesHint.textContent = "";
    return;
  }

  for (const item of items) {
    const entry = document.createElement("li");
    entry.className = "app-issues__item";
    entry.textContent = item;
    entry.setAttribute("dir", "auto");
    entry.title = item;
    elements.issuesList.append(entry);
  }

  elements.issuesBadge.textContent = title;
  elements.issuesHint.textContent = hint;
  elements.issuesPanel.hidden = false;
}
