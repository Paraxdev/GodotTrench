---
source: 12dfe8079d66dd10bb5ca7191aa90e4b3c01fe5b
---

# Cutscene com script { #scripted-cutscene }

O jogador entra numa sala, um aviso aparece, um guia sai andando, um barril explode, uma lâmpada acende e a saída
se abre. Nada disso precisa de código próprio: uma entidade temporizadora dispara uma saída por tempo, e cada saída é ligada
à entidade que deve agir. Você a encontra como o mapa `scripted_scene` no projeto de demonstração.

![O mapa scripted_scene no painel Lógica: start_zone dispara cutscene.start, as saídas step de cutscene vão para o guia, o barril e o aviso, e a saída broken do barril vai para hp_readout. A lâmpada e a porta de saída ficam mais abaixo, fora da imagem](../assets/cutscene/logic-graph.png)

## A linha do tempo { #the-timeline }

Toda entidade aqui vem com o pacote de entidades de gameplay. Instale-o antes se o seu projeto ainda não o tem, com
*Gameplay > Instalar entidades de gameplay*.

1. Cubra a entrada

    Desenhe um brush sobre a entrada e transforme-o em um `trigger_once` chamado `start_zone` (botão direito, *Criar entidade de brush >
    trigger_once*). Ele dispara `triggered` na primeira vez que o jogador entra e depois se desliga, então a cena
    toca só uma vez.

2. Adicione a linha do tempo

    Escolha `logic_sequence` em *Gameplay > Lógica*, ele cai no cursor 3D. Dê a ele o nome `cutscene`, defina `steps` como 5 e
    `interval` como 1.5. Depois de iniciado ele dispara `step_1`, `step_2` e assim por diante, com 1,5 segundo de intervalo.

    ![O menu Gameplay com Lógica aberto e logic_sequence destacado em Colocar no cursor](../assets/cutscene/menu-logic-sequence.png)

3. Inicie-a pelo trigger

    Vincule a saída `triggered` de `start_zone` a `cutscene.start`, com *Gameplay > Lógica > Vincular duas entidades selecionadas…*
    como em [Botão abre uma porta](button-door.md).

4. Ligue cada tempo

    Vincule cada saída step de `cutscene` à entidade que age naquele tempo, um par de cada vez. Arrastar de um pino de
    saída até um pino de entrada no [painel Lógica](../gameplay/logic-graph.md) também funciona.

| Saída | Alvo e entrada | O que acontece |
| --- | --- | --- |
| `step_1` | `hint.show` | Um `game_text` lê "Follow the guide" |
| `step_2` | `guide.start` | Um `npc_walker` parte ao longo da sua cadeia de `path_corner`, uma fileira de marcadores em que cada um nomeia o próximo |
| `step_3` | `barrel.ignite` | Um `prop_physics` explosivo queima o pavio, depois explode e se quebra |
| `step_4` | `lamp.turn_on` | A lâmpada acende |
| `step_5` | `exit_door.open` | A porta de saída desliza para cima |

Selecione `cutscene` e você deve ver as cinco em *Saídas* no Inspetor dele.

![O Inspetor do logic_sequence chamado cutscene: steps 5, interval 1.5 e Saídas (I/O, 5) começando com step_1 para hint.show](../assets/cutscene/inspector-sequence.png)

A sequência espera antes de cada passo, inclusive o primeiro, então o passo 1 dispara em 1,5 s e o passo 5 em 7,5 s. Para um
ritmo irregular, defina `times` com uma espera por passo, como `0.5 2 1 1 3`.

## Lendo a vida do jogador { #reading-the-players-health }

Algumas reações precisam de um pouco de lógica que nenhuma entrada embutida cobre. Um `logic_script` roda um trecho curto de GDScript quando
é disparado. Vincule a saída `broken` do barril a `hp_readout.run`, um `logic_script` cujo `source` escreve a
vida do jogador em um `game_text` de HUD chamado `hpmsg`:

```gdscript
var players = io.find_targets(this, "!player", null)
if players.is_empty():
	return
var hp = int(players[0].health)
for label in io.find_targets(this, "hpmsg", null):
	label.set_text("HP " + str(hp))
	label.show()
```

Use `!player` aqui, não `!activator`. `!activator` é quem deu início à cadeia, e as saídas step e `broken`
não carregam ninguém. `!player` sempre encontra o nó do grupo `player`. `broken` dispara depois que a explosão já causou o
dano, então a vida já está atualizada.

## Jogando { #playing-it }

O script da cena,
[godot/demo/scenes/scripted_scene.gd](https://github.com/Paraxdev/TrenchDot/blob/main/godot/demo/scenes/scripted_scene.gd),
só monta o mapa, faz o jogador aparecer no `info_player_start` e adiciona um `TrenchDotDebugOverlay`. Aperte **F3**
no jogo para ver cada saída disparar. É o jeito mais rápido de achar um passo ligado ao nome errado.

<div class="td-mcp" markdown>

## MCP { #mcp }

O mapa de demonstração é montado controlando o editor pelo [servidor MCP](../mcp.md). As chamadas estão em
[examples/mcp/scripted_scene.json](https://github.com/Paraxdev/TrenchDot/blob/main/examples/mcp/scripted_scene.json).

</div>
