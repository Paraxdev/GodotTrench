---
source: 978a2cda95dff6236396e32b6d494bed11f3f94b
---

# Spawners e caminhos { #spawners-and-paths }

> [!NOTE]
> Estas entidades vêm com o pacote de entidades de gameplay, veja
> [Instalando e personalizando](README.md#installing-and-customizing).

## info_spawner { #info_spawner }

Gera uma cena, como um inimigo ou um item, em pontos aleatórios dentro de `radius`. Dispare `spawn` de um trigger para uma
emboscada, ou defina um `interval` e dê `start` para um fluxo constante. Para espalhar os spawns por uma sala, use
[`trigger_spawn_area`](triggers.md#trigger_spawn_area).

| Key | Padrão | O que faz |
| --- | --- | --- |
| `scene` | | A cena `res://` a gerar |
| `count` | 1 | Quantas gerar a cada vez |
| `max_alive` | 5 | Nunca mais que isso vivas ao mesmo tempo, os spawns extras são pulados |
| `total` | 0 | Para de vez depois de tantas, 0 é ilimitado |
| `interval` | 0 | Segundos entre spawns enquanto o timer roda. 0 gera só em `spawn` |
| `radius` | 64 | A que distância do spawner elas podem aparecer. Arraste a alça dele na viewport |
| `start_active` | 0 | Roda o timer desde o carregamento do mapa |
| `spawn_on_ready` | 0 | Gera uma vez quando o mapa carrega |
| `spawn_group` | enemies | Grupo do Godot adicionado a todo nó gerado |
| `snap_to_ground` | 1 | Pousa cada spawn no chão abaixo dele |

* **Inputs:** `spawn`, `start`, `stop`, `toggle`, `kill_all`
* **Outputs:** `spawned(node)`, `all_dead`, `exhausted` (uma vez, na primeira vez que `total` é alcançado)

`all_dead` dispara quando todo nó gerado sumiu, para "limpe a sala para abrir a porta". Os nós gerados não têm
targetname, chegue neles pelo `spawn_group`, como `@enemies`.

Com as keys padrão ele não gera nada até algo disparar `spawn` nele. Para spawns sem trigger, defina
`spawn_on_ready`, ou um `interval` com `start_active`. O spawn só acontece no jogo em execução, no editor do Godot
o spawner continua sendo um nó vazio.

## info_teleport_destination { #info_teleport_destination }

Marca para onde `trigger_teleport` manda os corpos. Aponte-o no sentido em que o jogador deve olhar ao chegar, o yaw dele vira a
direção do olhar deles. A única key dele é `targetname`.

## path_corner { #path_corner }

Uma parada na rota de um `func_train` ou `npc_walker`, com `target` nomeando a próxima. Clique uma cadeia pronta com a
ferramenta **Caminho** (Shift+P). Ele dispara `reached` quando um trem ou caminhante chega, para um som ou evento naquela parada.

* **Outputs:** `reached(train)`
* **Keys:** `target`, `wait` 0 (segundos de espera aqui), `speed` 0 (nova velocidade do trem daqui em diante, 0 mantém a atual)

`speed` só afeta trens, os caminhantes mantêm a velocidade própria.

## npc_walker { #npc_walker }

Um ator com script que anda por uma cadeia de path corners tocando animações, para cutscenes e não para IA, como um guarda
passando por uma janela. Ele se move em linhas retas, ignorando gravidade e paredes, então posicione os cantos onde ele possa andar.

| Key | Padrão | O que faz |
| --- | --- | --- |
| `model` | | Uma cena que dá a ele uma malha e um `AnimationPlayer` |
| `target` | | O primeiro path corner para onde andar |
| `speed` | 2.0 | Metros por segundo |
| `loop` | 0 | Volta ao primeiro canto depois do último |
| `start_active` | 0 | Anda desde o carregamento do mapa |
| `walk_anim`, `idle_anim` | walk, idle | Animações tocadas enquanto anda e enquanto está parado |
| `face_travel` | 1 | Vira para onde anda |
| `collision` | default | A forma construída a partir do modelo, para os outros corpos esbarrarem nele, veja [Colisão](collision.md) |
| `collision_layer`, `collision_mask` | vazio | Números de camada como `1, 3`, vazio mantém o padrão do projeto |

* **Inputs:** `start`, `stop`, `walk_to(corner)`, `play_anim(name)`, `face(target)`
* **Outputs:** `arrived(corner)`, `reached_goal`, `finished` (os dois no último canto quando não há loop)

`face` e `walk_to` aceitam qualquer alvo, como `!player`. `walk_to` abandona o trecho atual e segue direto para lá.

Ele só carrega seu `model` no jogo em execução, então no editor do Godot é um nó vazio.
