---
source: c6245792b90999ca4b48812eb3d96e7a9c2389d6
---

# Matemática e valores { #math-and-values }

> [!NOTE]
> Estas entidades vêm com o pacote de entidades de gameplay, veja
> [Instalando e personalizando](README.md#installing-and-customizing).

Entidades de ponto invisíveis que guardam, calculam e comparam números, para que uma pontuação, uma contagem ou um preço possam guiar o mapa
sem um script. Elas são ligadas como toda entidade. Um número viaja por uma conexão como o valor de um output, como
`changed(value)` ou `result(value)`, e um input sem parâmetro próprio o recebe, veja
[Sem parâmetro](../parameters.md#no-parameter). Um parâmetro digitado na conexão vence o valor.

Números inteiros continuam inteiros, `2 + 3` é `5` e não `5.0`, e um número que chega como texto, como `2.5`, conta como número.
Tudo que não é número conta como texto.

## math_value { #math_value }

Guarda um número ou um pedaço de texto, do jeito que uma variável faz. Os botões dão `add` nele, e `changed` avisa o resto do
mapa qual é o valor agora. Ele só dispara `changed` quando o valor é diferente do anterior.

* **Inputs:** `set_value(value)`, `add(amount)` (1 por padrão), `get_value`, `reset`
* **Outputs:** `value(value)`, `changed(value)`
* **Keys:** `start_value` 0, um número ou texto

`add` soma dois números e concatena quando um dos dois é texto, então somar `!` a `hello` dá `hello!`. `get_value` dispara
`value` com o que está guardado, para quando algo precisa do número sem esperar uma mudança. `reset` volta para
`start_value`. Um `set_value` sem valor mantém o que está guardado.

## math_calc { #math_calc }

Faz um cálculo com o número que recebe e dispara a resposta. Ele não guarda nada, então a mesma entrada sempre dá
a mesma resposta. Encadeie vários para resolver algo em etapas.

* **Inputs:** `calculate(value)`, um valor ausente conta como 0
* **Outputs:** `result(value)`
* **Keys:** `operation` add, `operand` 1

| `operation` | `result` |
| --- | --- |
| `add`, `subtract`, `multiply` | o valor mais, menos ou vezes `operand` |
| `divide` | o valor dividido por `operand` |
| `modulo` | o resto da divisão, nunca negativo para um `operand` positivo, então `-1` módulo `3` é `2` |
| `power` | o valor elevado a `operand` |
| `min`, `max` | o menor ou o maior dos dois |
| `abs`, `negate` | o valor sem o sinal, ou com o sinal invertido |
| `round`, `floor`, `ceil` | o inteiro mais próximo (as metades se afastam de 0), arredondado para baixo, arredondado para cima |

`abs`, `negate`, `round`, `floor` e `ceil` ignoram `operand`. Quando não há resposta, como `divide` ou `modulo` por 0 ou
a raiz de um número negativo, nada dispara e o Godot imprime um aviso uma vez.

## math_compare { #math_compare }

Compara um valor guardado com `compare_value` e dispara de que lado ele ficou. `on_not_equal` dispara junto com `on_less`
e `on_greater`, para quando só importa uma diferença.

* **Inputs:** `set_value(value)`, `set_compare_value(value)`, `compare`, `set_and_compare(value)`
* **Outputs:** `on_less`, `on_equal`, `on_not_equal`, `on_greater`
* **Keys:** `compare_value` 0

`set_value` e `set_compare_value` só guardam, `compare` compara o que está guardado, e `set_and_compare` faz as duas coisas, então é
o input a ligar a um output `changed(value)`. Os números são comparados pelo valor, e `10` é maior que `9`. Dois textos
são comparados em ordem de dicionário, com letras maiúsculas antes das minúsculas. Decimais quase iguais, como `0.1 + 0.2` e `0.3`, contam como iguais.

## Exemplo: uma pontuação que abre uma porta { #example-a-score-that-opens-a-door }

Três botões de moeda e uma alavanca abrem a porta de um cofre quando há pelo menos três moedas.

1. Posicione um `math_value` chamado `score`, um `math_compare` chamado `enough` com `compare_value` 3 e um `logic_gate`
   chamado `door_gate` com `gate` and.
2. Ligue o `pressed` de cada botão de moeda a `score`, input `add`. Cada aperto soma 1.
3. Ligue o `changed` de `score` a `enough`, input `set_and_compare`. Ele repassa a nova pontuação.
4. Ligue o `on_less` de `enough` a `door_gate`, input `set_a`, parâmetro `false`. Ligue `on_equal` e `on_greater` do mesmo
   jeito com o parâmetro `true`. O `a` da porta lógica agora diz se há moedas suficientes.
5. Ligue o `pressed` da alavanca a `door_gate`, input `toggle_b`.
6. Ligue o `on_true` de `door_gate` à porta com o input `open`, e `on_false` com o input `close`.

A porta abre quando a terceira moeda cai com a alavanca puxada, ou o contrário. Puxar a alavanca de volta ou
zerar a pontuação a fecha. Para mostrar a pontuação, ligue o `changed` de `score` a um `game_text` com o input `set_text`, ou
ponha um `math_calc` no meio para mostrar pontos em vez de moedas.

O mapa logic playground constrói este salão e acrescenta uma loteria de luzes com `logic_random` e `logic_case` e um interruptor de lâmpada
com `logic_flipflop`. Abra `godot/demo/maps/logic_playground.tdm` no projeto de demonstração, ou veja como ele é feito em
[`examples/mcp/logic_playground.json`](https://github.com/Paraxdev/TrenchDot/blob/main/examples/mcp/logic_playground.json).

<div class="td-mcp" markdown>

## MCP { #mcp }

O script acima posiciona as entidades com `gameplay place`, uma chamada cada:

```json
{ "tool": "gameplay", "args": { "op": "place", "classname": "math_value", "origin": [-280, 40, 40],
  "properties": { "targetname": "score" },
  "outputs": [ { "output": "changed", "target": "enough", "input": "set_and_compare" } ] } }
{ "tool": "gameplay", "args": { "op": "place", "classname": "math_compare", "origin": [-280, 40, -40],
  "properties": { "targetname": "enough", "compare_value": "3" },
  "outputs": [ { "output": "on_greater", "target": "door_gate", "input": "set_a", "parameter": "true" } ] } }
```

</div>
