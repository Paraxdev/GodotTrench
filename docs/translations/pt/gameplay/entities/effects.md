---
source: 80f0288013e71dba5bd67c791a5e3e3272a71653
---

# Props, luzes e efeitos { #props-lights-and-effects }

> [!NOTE]
> Estas entidades vêm com o pacote de entidades de gameplay, veja
> [Instalando e personalizando](README.md#installing-and-customizing). `light`, `light_spot` e as sondas
> (`light_probe`, `env_reflection_probe` e `env_voxel_gi`) fazem parte do núcleo do addon e funcionam sem ele.

## prop_physics { #prop_physics }

Uma caixa ou barril que pode ser arremessado, danificado e quebrado. Ele quebra com 0 de vida, em um impacto mais rápido que
`impact_speed`, em `smash`, ou `fuse` segundos depois de `ignite`. Um prop explosivo explode quando quebra.

| Key | Padrão | O que faz |
| --- | --- | --- |
| `model` | | A cena com a aparência dele, só para a aparência |
| `size` | 16 16 16 | Meias extensões da caixa de colisão, então o padrão tem um metro de largura |
| `collision` | default | default e box usam `size`, convex hull e convex decomposition seguem o modelo, veja [Colisão](collision.md) |
| `collision_layer`, `collision_mask` | vazio | Números de camada como `1, 3`, vazio mantém o padrão do projeto |
| `mass` | 5.0 | Massa em quilogramas |
| `health` | 30 | Dano que ele aguenta antes de quebrar |
| `explosive` | 0 | Explode quando quebra |
| `explosion_radius`, `explosion_damage` | 192, 40 | Tamanho e força dessa explosão |
| `fuse` | 0.6 | Segundos que um prop aceso queima antes de quebrar |
| `impact_speed` | 0 | Quebra quando bate em algo mais rápido que isto, em m/s. 0 nunca quebra por impacto |
| `debris_scene` | | Uma cena gerada onde ele quebrou, como tábuas |

* **Inputs:** `smash`, `ignite(activator)`, `push(direction)`, `take_damage(amount, source)`
* **Outputs:** `damaged(hp)` com a vida que sobrou, `broken`

`push` recebe uma velocidade em unidades do mapa por segundo. `broken` dispara depois da explosão, então o que ele aciona já vê
o dano.

O prop constrói o modelo e a caixa de colisão dele toda vez que a cena carrega. O editor do Godot os mostra como uma prévia, que
não é salva com a cena.

## prop_model { #prop_model }

Um modelo estático com colisão opcional, para móveis, pedras e outros props que não devem se mover.

* **Keys:** `model` (.bbmodel, .glb, .gltf ou .tscn), `model_node`, `collision`, `collision_layer`,
  `collision_mask`, `scale` 1.0

`collision` default dá um casco convexo a cada malha, bom para formas quadradas. Escolha convex decomposition ou trimesh para
formas que o jogador atravessa ou pisa, como um arco, ou box para a forma mais barata, veja [Colisão](collision.md).
A colisão só é gerada quando o modelo não tem uma própria. Os modelos colocados com *Prop de modelo* e os modelos dentro de
prefabs são entidades prop_model, então colidem do mesmo jeito. Para um arquivo
que guarda várias variantes lado a lado, `model_node` mantém só o nó nomeado e o move para a origem do prop.

## env_explosion { #env_explosion }

Uma explosão com script, como uma carga em uma parede. Em `explode` ela chama `damage_method` em todo nó dentro de `radius` com
`damage * (1 - distance / radius)`, empurra corpos rígidos para longe e gera `effect_scene` para o clarão e o estrondo.

* **Inputs:** `explode(activator)`
* **Outputs:** `exploded`
* **Keys:** `radius` 256, `damage` 50, `force` 8 (m/s no centro, seja qual for a massa), `damage_method` take_damage,
  `effect_scene`

A distância é medida até a origem de cada nó, sem verificar linha de visão, então ela fere através das paredes.

## game_text { #game_text }

Mostra uma linha de texto no mundo, no HUD ou nos dois, para dicas, placas ou legendas de cutscene.

| Key | Padrão | O que faz |
| --- | --- | --- |
| `text` | | O texto a mostrar |
| `place` | world | `world` flutua na entidade, `hud` mostra na tela, `both` faz os dois |
| `start_visible` | 0 | Mostrado desde o carregamento do mapa |
| `duration` | 0 | Esconde depois desse tanto de segundos, 0 mantém até `hide` |
| `text_color` | 255 255 255 | Cor do texto |
| `world_size` | 0.01 | Tamanho do rótulo no mundo em metros por pixel |
| `billboard` | 1 | O rótulo no mundo encara a câmera e aparece através das paredes. Desligue para placas, que então mantêm os ângulos da entidade e ficam escondidas atrás das paredes |

* **Inputs:** `show`, `hide`, `set_text(text)`, `flash(seconds)`
* **Outputs:** `shown`, `hidden`

`show` e `hide` ligam e desligam o rótulo, não o nó. `flash` o mostra pelo tempo do parâmetro, em segundos, senão `duration`,
senão 2.

## light e light_spot { #light-and-light_spot }

Luzes omni e spot que os inputs ligam e desligam, para interruptores, alarmes e lâmpadas piscando. `switched` só
dispara quando o estado realmente muda. Elas fazem parte do núcleo do addon, então todo projeto as tem.

* **Inputs:** `turn_on`, `turn_off`, `toggle`
* **Outputs:** `switched(on)`
* **Keys:** `light_energy` 1.0 (1.5 para spots), `light_color` 255 255 255, `start_on` 1, `shadows` 0 (custa mais para
  renderizar), `fixture`, `fixture_off` hide, e `omni_range` 10, ou `spot_range` 15 e `spot_angle` 35 (a metade do
  ângulo do cone, em graus), `bake_mode` auto, `light_size` 0, `light_indirect_energy` 1, e `omni_attenuation` 1, ou
  `spot_attenuation` 1 e `spot_angle_attenuation` 1. Estas últimas keys definem as propriedades de luz do Godot de mesmo
  nome, e o bake também as usa

**Fixtures** mantêm o modelo da lâmpada em sincronia com a luz. Dê um targetname à geometria brilhante da lâmpada e ponha-o em
`fixture`, um `*` no fim combina um prefixo, então `hall_tube_*` cobre todos os tubos. Ela aparece enquanto a luz está ligada e
some enquanto está desligada, sem ligações. Com `fixture_off` em `dark` ela continua visível com a emissão desligada. `fixture`
aceita também as outras formas de [alvo](../parameters.md#targets), `@lamps` para um grupo de nós e `/root/...` para um caminho
de nó. O editor mostra uma prévia de todas as formas menos o caminho de nó, que só o Godot consegue resolver.

`bake_mode` e `light_size` decidem como [Pré-calcular iluminação](../../editor/light-baking.md#light-keys) trata a luz. Com
`auto` uma luz que o I/O pode ligar e desligar, uma com targetname ou `start_on` 0, continua em tempo real.

## env_sound { #env_sound }

Um som que toca de um ponto do nível, para alarmes, maquinário ou uma batida ligada a um evento.

* **Inputs:** `play`, `stop`, `toggle`
* **Outputs:** `finished`
* **Keys:** `sound` (um arquivo de áudio `res://`), `volume_db` 0, `max_distance` 0 (sem limite), `autoplay_on` 0 (toca desde
  o carregamento do mapa), `loop_sound` 0

## env_particles { #env_particles }

Um efeito de partículas para fogo, fumaça, faíscas, poeira e chuva, com visuais embutidos para você não precisar construir um.
`burst` emite uma rajada, e depois um efeito em execução continua.

* **Inputs:** `start`, `stop`, `toggle`, `burst`
* **Outputs:** `finished`
* **Keys:** `amount` 32 (partículas vivas ao mesmo tempo), `lifetime` 2.0, `one_shot` 0, `start_emitting` 0, `effect` rise,
  `color` 255 255 255, `size` 0, `blend` auto, `area` 10 1 10

`effect` é `rise` (poeira brilhante ou brasas), `fire`, `smoke`, `sparks` ou `rain` (cai de uma caixa `area` de metros
de largura, altura e profundidade). `color` tinge o efeito, e branco mantém as cores do próprio efeito. `size` define o tamanho da partícula em
metros, 0 usa o do próprio efeito. `blend` é `add` para brilho ou `mix` para fumaça. Os padrões de cada efeito ficam bons
sozinhos, então comece só com `effect`.

## infodecal { #infodecal }

Um decal projetado, uma caixa que joga um material sobre tudo que está dentro dela, para manchas que contornam degraus, canos ou
terreno irregular. [Decals projetados](../../editor/projected-decals.md) mostra como posicionar um e com o que tomar cuidado.

* **Keys:** `material` (um nome como `base/stain` ou um caminho `res://`), `texture`, `normal_texture`, `size` 64 32 64,
  `modulate` 255 255 255

`material` traz as texturas e a cor junto, `texture` e `normal_texture` são imagens `res://` usadas sem um material.
`size` é a largura, a profundidade e a altura da caixa em unidades do mapa. Se ficar em 64 32 64, o
[tamanho de textura](../../godot/materials.md#texture-size) do material define a largura e a altura no lugar. `modulate` tinge o decal.

## light_probe, env_reflection_probe e env_voxel_gi { #light_probe-env_reflection_probe-and-env_voxel_gi }

Sondas que iluminam e refletem objetos em movimento, veja [Iluminação pré-calculada no Godot](../../godot/baked-lighting.md). Nenhuma tem
inputs ou outputs.

| Entidade | O que é | Keys |
| --- | --- | --- |
| `light_probe` | Um ponto que adiciona uma sonda de luz ao bake, para lugares que a grade de sondas não alcança | Nenhuma |
| `env_reflection_probe` | Um volume brush que vira um `ReflectionProbe` do tamanho do brush | `intensity` 1.0, `update` once, `box_projection` 1, `interior` 0, `max_distance` 0 |
| `env_voxel_gi` | Um volume brush que vira um `VoxelGI` do tamanho do brush, pré-calculado no editor do Godot a cada construção | `subdiv` 128, `energy` 1.0, `bake_on_build` 1 |

`update` em `always` captura uma sonda de reflexo a cada frame para objetos em movimento, o que custa muito. `box_projection`
alinha os reflexos às paredes de uma sala, `interior` deixa o céu de fora para salas sem janelas, e
`max_distance` é quantos metros a sonda enxerga além da caixa. `subdiv` é 64, 128, 256 ou 512 voxels ao longo do
lado mais longo, mais é mais nítido e mais lento de pré-calcular.
