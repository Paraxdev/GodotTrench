---
source: 414dc5a3711ba57511bd4ecc18dc19f159af44d7
---

# Guia rápido de I/O { #io-cheatsheet }

Um resumo de uma página de [Como funciona o I/O de entidades](io.md) e de [Alvos, parâmetros e inputs](parameters.md), para quando
você já conhece a ideia e só precisa dos detalhes.

## Quando uma saída dispara { #when-an-output-fires }

O sinal dispara, conta contra *Vezes*, espera o *Atraso*, depois chama a entrada em todo nó que o alvo encontrar. Se não encontrar
nenhum, nada acontece, e um targetname ou caminho de nó que não encontra nada avisa uma vez.

## Alvos { #targets }

| Escreva | Alcança |
| --- | --- |
| `main_door` | Toda entidade com esse targetname, no mesmo mapa |
| `door_*` | Todo targetname que começa com `door_` |
| `@enemies` | Todo nó desse grupo do Godot, em qualquer lugar da árvore |
| `!player` | O primeiro nó do grupo `player` |
| `!activator` | Quem iniciou a cadeia, `!caller` significa o mesmo |
| `!self` | A entidade dona da saída |
| `/root/Game` | O nó nesse caminho absoluto |

## Parâmetros { #parameters }

Um valor simples é tentado como int, float, bool, depois `x y z` como um `Vector3`, senão continua uma string. Um JSON `[a, b]` se
espalha em vários argumentos. `$activator`, `$self`, `$position` e `$caller_name` são preenchidos na entidade **que recebe**.

Sem parâmetro, o ativador vai para o primeiro argumento de objeto, e os outros valores que a saída carregava preenchem o resto em
ordem.

## Quem mantém o ativador { #who-keeps-the-activator }

| Sinais | Ativador |
| --- | --- |
| `triggered`, `pressed`, `entered`, `exited`, `hurt`, `teleported`, `pushed` | Repassado sem mudança |
| `arrived(corner)`, `spawned(node)`, `reached(train)` | Substituído pelo nó entre parênteses |
| `opened`, `timer`, `step_N`, `on_true`, `on_case_N`, `result`, `broken`, `switched`, `map_spawn` | Descartado |

Se a cadeia o perdeu e você precisa do jogador, use `!player` como alvo.

## Entradas que todo nó tem { #inputs-every-node-has }

`kill`, `show`, `hide`, `enable`, `disable` e `toggle` funcionam em qualquer nó, desde que a entidade não tenha um método com esse
nome. Uma entrada que combina com uma propriedade a define com o parâmetro.

## Escopo do logic_script { #logic_script-scope }

Um [`logic_script`](entities/scripting.md) roda um trecho de GDScript quando disparado. Dentro dele, `this` é o próprio logic_script,
`activator` o nó que iniciou a cadeia, `parameter` o valor passado, `io` os auxiliares do `TrenchDotIO` e `tree` a `SceneTree`.
Indente com tabs.

```gdscript
var p = io.find_targets(this, "!player", null)
if not p.is_empty():
	p[0].take_damage(25, this)
```

## Nada aconteceu? { #nothing-happened }

Confira o painel Problemas, depois o Grafo lógico, depois F3 no jogo, depois registre em log `TrenchDotIO.events()`. Veja
[Depurando as conexões](debugging.md).
