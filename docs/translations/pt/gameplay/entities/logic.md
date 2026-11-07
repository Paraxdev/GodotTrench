---
source: 5168ac40e48ba63850a1be1795dae420bfe6bf5f
---

# Lógica { #logic }

> [!NOTE]
> Estas entidades vêm com o pacote de entidades de gameplay, veja
> [Instalando e personalizando](README.md#installing-and-customizing).

Entidades de ponto invisíveis que decidem quando as coisas acontecem. Ficam entre um trigger ou botão e o que ele afeta,
e moldam o sinal no caminho. As entidades que trabalham com números estão em [Matemática e valores](math.md), e as que executam
código ou animações em [Scripts e chamadas](scripting.md).

## logic_relay { #logic_relay }

Repassa `trigger` como `triggered` enquanto ativado. Ligue várias coisas a um relay e depois dê `disable` nele para desligar a
cadeia inteira de uma vez.

* **Inputs:** `trigger(activator)`, `enable`, `disable`, `toggle`
* **Outputs:** `triggered(activator)`
* **Keys:** `start_disabled` 0

## logic_branch { #logic_branch }

Lembra verdadeiro ou falso e dispara `on_true` ou `on_false` em `test`, para verificações como "a porta só abre se a energia
está ligada". O gerador a define como verdadeiro, o botão da porta a testa.

* **Inputs:** `set_true`, `set_false`, `toggle`, `test`, `set_and_test(value)`
* **Outputs:** `on_true`, `on_false`
* **Keys:** `start_value` 0

## logic_gate { #logic_gate }

Guarda dois valores verdadeiro ou falso, `a` e `b`, e os combina com uma porta, para verificações como "a porta abre quando a
energia está ligada **e** a alavanca está puxada". Cada mudança de input que inverte o resultado dispara `changed(on)` e depois `on_true`
ou `on_false`, então ninguém precisa perguntar. `test` dispara de novo o resultado atual, para quando outra coisa precisa saber agora.

* **Inputs:** `set_a(value)`, `set_b(value)`, `toggle_a`, `toggle_b`, `test`
* **Outputs:** `on_true`, `on_false`, `changed(on)`
* **Keys:** `gate` and, `start_a` 0, `start_b` 0

| `gate` | Verdadeiro quando |
| --- | --- |
| `and` | os dois são verdadeiros |
| `or` | pelo menos um é verdadeiro |
| `xor` | exatamente um é verdadeiro |
| `nand` | os dois não são verdadeiros ao mesmo tempo |
| `nor` | nenhum é verdadeiro |
| `xnor` | os dois são iguais |
| `not` | `a` é falso, `b` é ignorado |

`set_a` e `set_b` sem valor definem como verdadeiro, então o output `entered` de um trigger ligado a `set_a` liga `a`. Um número
é verdadeiro a menos que seja 0, e um texto é verdadeiro só para `true`, `yes` ou `1`. Os valores iniciais não disparam nada, nem
uma mudança que deixa o resultado como estava.

## logic_flipflop { #logic_flipflop }

Alterna toda vez que é disparado, então um botão só pode ligar e desligar algo de novo. `on_a` dispara quando ele liga
e `on_b` quando desliga. `reset` o devolve a `start_on` sem disparar nenhum dos dois, e dispara só `changed` se
isso foi uma mudança.

* **Inputs:** `trigger`, `reset`
* **Outputs:** `on_a`, `on_b`, `changed(on)`
* **Keys:** `start_on` 0 (ligado, então o primeiro trigger dispara `on_b`)

## logic_case { #logic_case }

Compara o valor que recebe com até oito casos e dispara o output do primeiro que combina, ou
`on_default` quando nenhum combina. Ele transforma um número, como o índice de um `logic_random` ou a pontuação de um `math_value`, em
eventos diferentes.

* **Inputs:** `in_value(value)`
* **Outputs:** `on_case_1` a `on_case_8`, `on_default`
* **Keys:** `case_1` a `case_8`, vazias

Os números combinam pelo valor, então o caso `2` combina com um valor 2 e com `2.0`. O texto precisa combinar exatamente, `Red` não é
`red`, e espaços em volta de um caso são ignorados. Casos vazios são pulados, o primeiro que combina vence, e um valor ausente
vai para `on_default`.

## logic_random { #logic_random }

Escolhe ao acaso um entre até oito outputs, ou rola uma chance. Ligue `out_1` a `out_3` a três luzes e `pick` acende
uma delas. `roll` decide por chance, por exemplo se um baú tem tesouro.

* **Inputs:** `pick`, `roll`
* **Outputs:** `out_1` a `out_8`, `picked(index)`, `on_success`, `on_fail`
* **Keys:** `count` 3 (1 a 8), `no_repeat` 0, `chance` 50 (porcentagem)

`pick` dispara `picked` com o número do output escolhido e depois esse output, então `picked` pode alimentar um
[`logic_case`](#logic_case). Com `no_repeat` ele nunca escolhe o output que escolheu da última vez, a menos que `count` seja 1.
`roll` dispara `on_success` com probabilidade de `chance` por cento e `on_fail` no resto, então 0 nunca dá certo e 100
sempre dá.

## logic_counter { #logic_counter }

Conta entre `min` e `max`, para quebra-cabeças como "aperte os três botões": cada botão manda `add`, e `hit_max`
abre a porta. O valor é limitado e nada dispara a menos que ele mude. `hit_max` e `hit_min` disparam quando uma mudança
cai em um limite, `reset` dispara só `changed`.

* **Inputs:** `add(amount)` (1 por padrão), `subtract(amount)`, `set_value(value)`, `reset`
* **Outputs:** `hit_max`, `hit_min`, `changed(value)`
* **Keys:** `min` 0, `max` 3, `start_value` 0

## logic_timer { #logic_timer }

Dispara `timer` de novo e de novo, para qualquer coisa que se repete, como uma luz piscando ou ondas de inimigos. Ele espera
`interval` segundos entre os disparos e roda desde o carregamento do mapa, a menos que `start_on` seja 0. Com `random_max` acima de `random_min`
cada espera é aleatória nessa faixa, então as repetições parecem menos mecânicas. `once` o para depois do primeiro disparo, o que
o torna um atraso simples.

* **Inputs:** `start`, `stop`, `toggle`, `fire` (uma vez, agora)
* **Outputs:** `timer`
* **Keys:** `interval` 1.0, `random_min` 0, `random_max` 0, `start_on` 1, `once` 0

## logic_auto { #logic_auto }

Dispara `map_spawn` uma vez, depois que toda entidade do mapa está pronta. Ligue aqui tudo que roda desde o início, como música
ou um texto de introdução.

* **Outputs:** `map_spawn`

## logic_sequence { #logic_sequence }

Uma linha do tempo de cutscene. Ela dispara até oito passos numerados em ordem e espera antes de cada um, o primeiro incluído. Ligue
cada passo ao que acontece naquele momento, uma porta abrindo, uma linha de texto, um ator andando.

* **Inputs:** `start`, `stop`, `reset`
* **Outputs:** `step_1` a `step_8`, `step(index)`, `finished`
* **Keys:** `steps` 3 (1 a 8), `interval` 1.0, `times` vazia, `start_active` 0 (roda desde o carregamento do mapa), `loop` 0

`times` lista uma espera por passo, como `0 1 0.5`, e os passos sem uma usam `interval`. Uma sequência em loop nunca dispara
`finished`. `start` é ignorado enquanto ela roda, e depois de `stop` ela recomeça no passo 1. Veja o
[tutorial da cutscene](../../tutorials/cutscene.md).
