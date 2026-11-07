---
source: 59a13276848e83b80ec8fffb5e9295c0444e1a52
---

# Terreno e scatter { #terrain-and-scatter }

## Terreno { #terrain }

Um heightmap numa grade regular, uma altura por vértice da grade, pintado com até quatro camadas de textura. Os dados em massa são texto base64 no JSON e bytes brutos no [contêiner binário](container.md#value-encoding).

| Chave | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `origin` | `[x, y, z]` | obrigatória | Posição no mundo do primeiro vértice |
| `resolution` | `[nx, nz]` | obrigatória | Número de vértices ao longo de X e de Z, pelo menos 2 cada |
| `cell_size` | número | obrigatória | Unidades do mapa entre vértices vizinhos |
| `heights` | base64 | obrigatória | Floats de 32 bits little endian, uma altura por vértice |
| `layers` | array | obrigatória | Até quatro camadas de textura, veja abaixo |
| `splat` | base64 | omitida | Quatro bytes de peso por vértice, um por camada, dizendo quanto cada camada aparece ali |
| `holes` | base64 | omitida | Um byte por célula, diferente de zero recorta a célula do terreno |
| `chunk_cells` | inteiro | `32` | Sempre gravada. Células por lado de chunk. O terreno é dividido em chunks desse tamanho para renderização e colisão |

O vértice `(i, j)` fica em `origin + (i * cell_size, heights[j * nx + i], j * cell_size)`. O `splat` usa a mesma ordem, seus pesos são normalizados na leitura, e uma entrada toda zerada significa a camada 0. O `holes` cobre as `(nx - 1) * (nz - 1)` células na mesma ordem.

Cada camada:

| Chave | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `material` | string | obrigatória | Material pintado por esta camada |
| `tile` | número | `256` | Unidades do mapa por repetição da textura |
| `detile` | número | `0` | De 0 a 1, com que força a repetição é quebrada para o padrão de grade parar de aparecer (desladrilhamento) |
| `detile_sharpen` | número | `0.5` | De 0 a 1, quão nítido o resultado desladrilhado continua |

Veja [Camadas de terreno](../editor/terrain-layers.md) para saber como essas configurações aparecem na prática.

Os terrenos continuam alinhados aos eixos. Uma instância move o centro de um terreno por toda a sua transformação, rotação incluída, mas nunca gira a grade em si.

## Scatter { #scatter }

Um conjunto de scatter guarda, num só nó, muitas cópias de poucos modelos, como árvores, pedras ou grama pintadas nas superfícies. Veja [Scatter](../editor/scatter.md) para o lado do editor. Toda chave exceto `targets` e `material` é sempre gravada.

| Chave | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `name` | string | obrigatória | Nome mostrado no editor |
| `kind` | string | `props` | O que o Godot constrói. `props` mantém scripts e colisão, `foliage` é um MultiMesh sem colisão |
| `targets` | array de ids de nó | omitida | As superfícies em que o conjunto é pintado |
| `items` | array | obrigatória | Os modelos que ele espalha, veja abaixo |
| `collision` | string | `none` para foliage, senão `convex` | Forma de colisão por instância: `none`, `box`, `convex`, `decomposition` ou `trimesh` |
| `cast_shadows` | bool | `true` | Se as instâncias projetam sombras |
| `visibility_range` | número | `0` | Esconde as instâncias além desta distância, 0 é nunca |
| `chunk_size` | número | `0` | Divide o conjunto em células deste tamanho para o Godot poder descartar o que está fora da tela, 0 é uma célula só. Conjuntos novos usam 2048 |
| `static_props_multimesh` | bool | `false` | Desenha também como MultiMesh as cenas de prop que carregam scripts. Mais barato para muitos props, mas os scripts deles são descartados |
| `material` | string | omitida | Substituição de material para o conjunto todo |
| `instances` | array | vazio | As instâncias colocadas, veja abaixo |

Copie um conjunto junto com uma superfície que ele tem como alvo, e a cópia passa a ter como alvo a superfície copiada.

Cada item é um modelo que o conjunto pode colocar:

| Chave | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `source` | string | obrigatória | Modelo ou cena `res://` |
| `weight` | número | `1` | Chance relativa de este item ser escolhido |
| `scale` | `[min, max]` | `[0.8, 1.2]` | Intervalo de uma escala uniforme aleatória |
| `spacing` | número | `48` | Distância mínima até toda outra instância |
| `align` | número | `0` | 0 fica em pé, 1 segue a normal da superfície, valores no meio se inclinam parcialmente |
| `random_yaw` | bool | `true` | Gira cada instância para uma direção aleatória |
| `tilt` | número | `0` | Maior inclinação aleatória em graus |
| `sink` | número | `0` | Unidades do mapa afundadas na superfície, para raízes e bases de pedra não flutuarem |
| `material` | string | omitida | Substituição de material para este item. Vale mais que a do conjunto |
| `enabled` | bool | `true`, omitida | Se o pincel pinta este item. As instâncias já colocadas ficam |

Cada instância é um array simples `[item, x, y, z, pitch, yaw, roll, scale]`, em que `item` indexa `items` e os ângulos estão em graus na ordem YXZ. Ao salvar, a posição é arredondada para 0,01, os ângulos para 0,1 e a escala para 0,001, o que deixa o contêiner guardá-los como [colunas de inteiros](container.md#value-encoding).
