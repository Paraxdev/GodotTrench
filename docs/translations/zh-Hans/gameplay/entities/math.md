---
source: c6245792b90999ca4b48812eb3d96e7a9c2389d6
---

# 数学与数值 { #math-and-values }

> [!NOTE]
> 这些实体来自玩法实体包，见[安装与自定义](README.md#installing-and-customizing)。

不可见的点实体，用来存储、计算和比较数字，让分数、计数或价格不用脚本就能左右地图。它们的接线方式和其他实体一样。一个数字作为输出的值沿着连接传递，比如 `changed(value)` 或 `result(value)`，自己没有参数的输入会接收它，见[没有参数](../parameters.md#no-parameter)。连接里手写的参数比这个值优先。

整数会保持整数，`2 + 3` 是 `5` 而不是 `5.0`，以文字形式到来的 `2.5` 这样的内容也算数字。凡是不是数字的，都算文字。

## math_value { #math_value }

存储一个数字或一段文字，就像变量一样。按钮对它 `add`，`changed` 告诉地图的其余部分它现在是多少。只有值和之前不同时，它才会触发 `changed`。

* **输入：** `set_value(value)`、`add(amount)`（默认 1）、`get_value`、`reset`
* **输出：** `value(value)`、`changed(value)`
* **键：** `start_value` 0，一个数字或文字

`add` 把两个数字相加，只要有一个是文字就改为拼接，所以在 `hello` 后面加 `!` 得到 `hello!`。`get_value` 带着存储的内容触发 `value`，用于需要这个数字、又不想等它变化的时候。`reset` 回到 `start_value`。不带值的 `set_value` 会保持存储的内容不变。

## math_calc { #math_calc }

对收到的数字做一次计算，并触发答案。它什么也不保留，所以同样的输入永远得到同样的答案。把几个串起来，就能分步算出结果。

* **输入：** `calculate(value)`，缺少值时按 0 算
* **输出：** `result(value)`
* **键：** `operation` add，`operand` 1

| `operation` | `result` |
| --- | --- |
| `add`、`subtract`、`multiply` | 值加上、减去或乘以 `operand` |
| `divide` | 值除以 `operand` |
| `modulo` | 除法的余数，`operand` 为正时永远不为负，所以 `-1` 对 `3` 取模是 `2` |
| `power` | 值的 `operand` 次方 |
| `min`、`max` | 两者中较小的或较大的 |
| `abs`、`negate` | 去掉符号的值，或符号翻转后的值 |
| `round`、`floor`、`ceil` | 最接近的整数（一半的情况远离 0）、向下取整、向上取整 |

`abs`、`negate`、`round`、`floor` 和 `ceil` 会忽略 `operand`。算不出答案时，比如 `divide` 或 `modulo` 除以 0，或者对负数开方，什么也不会触发，Godot 只打印一次警告。

## math_compare { #math_compare }

把存储的值和 `compare_value` 比较，并触发它落在哪一边。`on_not_equal` 会和 `on_less`、`on_greater` 一起触发，用于只关心有没有差别的时候。

* **输入：** `set_value(value)`、`set_compare_value(value)`、`compare`、`set_and_compare(value)`
* **输出：** `on_less`、`on_equal`、`on_not_equal`、`on_greater`
* **键：** `compare_value` 0

`set_value` 和 `set_compare_value` 只负责存储，`compare` 比较已存储的内容，`set_and_compare` 两件事一起做，所以它是接给 `changed(value)` 输出的那个输入。数字按值比较，`10` 比 `9` 大。两段文字按字典顺序比较，大写字母排在小写字母前面。几乎相等的小数，比如 `0.1 + 0.2` 和 `0.3`，算作相等。

## 示例：用分数打开一扇门 { #example-a-score-that-opens-a-door }

三个金币按钮和一根拉杆，在至少投入三枚金币后打开一扇金库门。

1. 放置一个叫 `score` 的 `math_value`，一个叫 `enough`、`compare_value` 为 3 的 `math_compare`，以及一个叫 `door_gate`、`gate` 为 and 的 `logic_gate`。
2. 把每个金币按钮的 `pressed` 接到 `score`，输入选 `add`。每按一次加 1。
3. 把 `score` 的 `changed` 接到 `enough`，输入选 `set_and_compare`。它会把新的分数传过去。
4. 把 `enough` 的 `on_less` 接到 `door_gate`，输入选 `set_a`，参数填 `false`。`on_equal` 和 `on_greater` 也一样接，参数填 `true`。现在门控的 `a` 就表示金币够不够。
5. 把拉杆的 `pressed` 接到 `door_gate`，输入选 `toggle_b`。
6. 把 `door_gate` 的 `on_true` 接到门，输入选 `open`，`on_false` 接过去，输入选 `close`。

第三枚金币落下时如果拉杆已经拉下，门就会打开，反过来先投币后拉杆也一样。把拉杆拉回去，或者重置分数，门就会关上。想显示分数，就把 `score` 的 `changed` 接到一个 `game_text` 的 `set_text` 输入，或者中间放一个 `math_calc`，让它显示点数而不是金币。

逻辑游乐场地图搭了这个大厅，还加了一个用 `logic_random` 和 `logic_case` 做的灯光抽奖，以及一个用 `logic_flipflop` 做的灯开关。在演示项目里打开 `godot/demo/maps/logic_playground.tdm`，或者去看它是怎么做出来的：[`examples/mcp/logic_playground.json`](https://github.com/Paraxdev/TrenchDot/blob/main/examples/mcp/logic_playground.json)。

<div class="td-mcp" markdown>

## MCP { #mcp }

上面的脚本用 `gameplay place` 放置这些实体，每次调用放一个：

```json
{ "tool": "gameplay", "args": { "op": "place", "classname": "math_value", "origin": [-280, 40, 40],
  "properties": { "targetname": "score" },
  "outputs": [ { "output": "changed", "target": "enough", "input": "set_and_compare" } ] } }
{ "tool": "gameplay", "args": { "op": "place", "classname": "math_compare", "origin": [-280, 40, -40],
  "properties": { "targetname": "enough", "compare_value": "3" },
  "outputs": [ { "output": "on_greater", "target": "door_gate", "input": "set_a", "parameter": "true" } ] } }
```

</div>
