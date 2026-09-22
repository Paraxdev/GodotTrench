// A tiny hyperscript helper so the rest of the app can build DOM declaratively instead of
// repeating createElement, className, setAttribute, appendChild and addEventListener by hand.
//
//   el("button", { class: "btn", title: "Save", on: { click: save } }, icon("save"))
//   el("div", { class: "row" }, [labelEl, valueEl])
//
// props keys:
//   class / className   sets node.className
//   text                sets node.textContent (safe, no markup)
//   html                sets node.innerHTML (only for content already sanitized)
//   style               object merged into node.style
//   dataset             object merged into node.dataset
//   on                  object of event name to handler, wired with addEventListener
//   value               always assigned as a property, so textarea and input both work
//   boolean values      assigned as a property (controls, hidden, spellcheck, ...)
//   anything else        setAttribute (type, title, role, aria-*, src, href, ...)
//
// children may be a node, a string, null/false (skipped), or an array of the same, nested.
// This is HTML only. SVG still uses createElementNS through the svgEl helper in cards.js.

export function el(tag, props, children) {
  const node = document.createElement(tag);
  if (props) setProps(node, props);
  if (arguments.length > 2) append(node, children);
  return node;
}

function setProps(node, props) {
  for (const key in props) {
    if (!Object.prototype.hasOwnProperty.call(props, key)) continue;
    const value = props[key];
    if (value == null) continue;
    if (key === "class" || key === "className") node.className = value;
    else if (key === "text") node.textContent = value;
    else if (key === "html") node.innerHTML = value;
    else if (key === "style") Object.assign(node.style, value);
    else if (key === "dataset") Object.assign(node.dataset, value);
    else if (key === "on") {
      for (const type in value) {
        if (Object.prototype.hasOwnProperty.call(value, type)) node.addEventListener(type, value[type]);
      }
    } else if (key === "value" || typeof value === "boolean") {
      try {
        node[key] = value;
      } catch (err) {
        if (value !== false) node.setAttribute(key, String(value));
      }
    } else {
      node.setAttribute(key, value);
    }
  }
}

function append(node, children) {
  if (children == null || children === false) return;
  if (Array.isArray(children)) {
    children.forEach((child) => append(node, child));
    return;
  }
  node.appendChild(children instanceof Node ? children : document.createTextNode(String(children)));
}
