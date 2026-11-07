---
source: 7e5d26a9fedbc9f43e9edcb92409992b19d24676
---

# Navegação { #navigation }

O `TrenchDotNav` pré-calcula uma malha de navegação a partir de um mapa construído, para que inimigos e NPCs achem o caminho por ele.

```gdscript
map.build()
var region := await TrenchDotNav.bake_region(map)
```

O `bake_region` faz o bake em uma thread de trabalho, adiciona um `NavigationRegion3D` chamado `navigation` sob o mapa e só retorna quando os caminhos sobre ele já funcionam. Passe um nó que guarda vários mapas para fazer o bake de todos em uma única malha. Uma reconstrução libera a região, então faça o bake de novo depois dela.

## O que conta { #what-counts }

O bake lê a colisão do mapa, não as malhas, então tudo em que um jogador pode esbarrar é o que um agente contorna.

| No mapa | Na malha de navegação |
| --- | --- |
| Brushes do mundo, `func_detail`, terreno e colisão de props | Sim |
| Brushes de clip | Sim, eles bloqueiam como paredes |
| Brushes ilusórios, triggers e props de física | Não |
| Portas, botões, plataformas e trens | Não, então uma porta fechada não sela o vão dela |
| Corpos no grupo `navigation_ignore` | Não |

Isso quer dizer que os caminhos atravessam portas fechadas, então um agente precisa abrir a porta em que esbarra.

## Tamanho do agente { #agent-size }

`TrenchDotNav.profile(radius, height, max_climb, max_slope)` devolve as configurações para um tamanho de agente. O raio mantém os caminhos longe das paredes, a altura descarta tetos baixos, *max_climb* é o degrau mais alto que o agente sobe e *max_slope* a rampa mais íngreme. Os tamanhos são em metros, a inclinação em graus:

| Argumento | Padrão |
| --- | --- |
| `radius` | 0.3 |
| `height` | 1.8 |
| `max_climb` | 0.3 |
| `max_slope` | 45 |

A malha usa células de 0,1 m e ajusta os tamanhos a elas. Passe um perfil como segundo argumento, por exemplo para uma criatura larga e baixa:

```gdscript
var region := await TrenchDotNav.bake_region(map, TrenchDotNav.profile(0.5, 1.2))
```

(O `bake_region` define o tamanho de célula do mapa de navegação como o do bake, e outras regiões nesse mapa precisam usar o mesmo.)

## Cache { #caching }

O primeiro bake de um mapa grande leva alguns segundos. O resultado fica em cache em `user://trenchdot_navigation`, com chave feita da colisão e das configurações, então a próxima inicialização com o mesmo mapa o carrega em uma fração de segundo. Passe `false` como terceiro argumento para fazer o bake sem o cache.

## Conferindo um mapa { #checking-a-map }

`TrenchDotNav.bake(map)` faz o bake na thread que o chamou e devolve a `NavigationMesh`, útil para ferramentas e testes. `TrenchDotNav.islands(nav)` lista as partes desconexas dela como índices de polígonos, a maior primeiro.

Uma sala que deveria ser alcançável mas fica numa ilha só dela costuma ter um vão bloqueado por algo baixo, ou um degrau mais alto que `max_climb`. Topos de parede e o piso dentro de paredes grossas também aparecem como ilhas pequenas.

> [!WARNING]
> O Godot interrompe a busca de caminho depois de 4096 polígonos e devolve o caminho até ali, o que num mapa grande parece um ponto inalcançável. `TrenchDotNav.path(node, from, to)` não tem limite. Para os agentes, defina `path_search_max_polygons` como 0 no `NavigationAgent3D`.

## Vendo no TrenchDot { #seeing-it-in-trenchdot }

Você vê por onde um agente anda direto no TrenchDot, sem rodar o jogo.

1. Abra a cena no Godot

    Salve o mapa dentro do projeto e abra o projeto no editor do Godot com uma cena cujo `FuncGodotMap` constrói esse mapa.

2. Ligue a área caminhável

    Vá em *Exibir > Área caminhável > Mostrar área caminhável*. O tamanho do agente fica logo abaixo, e a linha de baixo diz quanto é caminhável e quantas ilhas estão isoladas.

    ![O menu Exibir com o submenu Área caminhável aberto: Mostrar área caminhável, depois Raio 0.30 m, Altura 1.80 m, Subida máx. 0.30 m, Inclinação máx. 45.00 graus e Ainda não pré-calculado](../assets/navigation/menu-walkable-area.png)

O Godot faz o bake da cena pelo [Live Link](live-link.md) e o TrenchDot desenha o resultado nas vistas 3D e 2D. (Se ele não conseguir fazer o bake, a barra de status diz o motivo, por exemplo que o mapa precisa ser salvo primeiro dentro do projeto.)

As cores significam:

| Cor | Significado |
| --- | --- |
| Verde azulado | A maior área conectada, onde a maioria dos agentes vai estar |
| Vermelhão | Ilhas que um agente na área verde não alcança. Procure um degrau mais alto que a *Subida máx.*, um vão mais estreito que o dobro do *Raio* ou uma rampa mais íngreme que a *Inclinação máx.* |

A sobreposição faz o bake de novo um instante depois de cada edição. Sem o [modo ao vivo](live.md), o Godot constrói o mapa como ele está primeiro, como em *Construir no Godot*.

> [!TIP]
> Topos de parede e de telhado aparecem como ilhas pequenas em vermelhão. Procure piso vermelhão ao lado de piso verde.

<div class="td-mcp" markdown>

## MCP { #mcp }

O [servidor MCP](../mcp.md) deixa agentes e scripts controlarem o editor. Eles conferem um mapa com a ferramenta `walkability`, que devolve as ilhas com suas áreas e limites em unidades do mapa.

</div>
