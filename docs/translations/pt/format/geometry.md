---
source: d8695a266e0b8802237d74d9393aab6b51f516c4
---

# Geometria { #geometry }

## Brush { #brush }

Um sólido convexo, guardado como seus cantos exatos mais uma lista de faces que os indexam, então você obtém os cantos sem precisar intersectar planos. Os planos das faces são recalculados ao carregar.

| Chave | Tipo | Significado |
| --- | --- | --- |
| `vertices` | array de `[x, y, z]` | Cantos em unidades do mapa |
| `faces` | array de faces | Um polígono por face |

Cada face:

| Chave | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `indices` | array de inteiros | obrigatória | Índices em `vertices`, pelo menos três, em sentido anti-horário visto de fora |
| `material` | string | `""` | Sempre gravada. Nome da textura, por exemplo `base/wall` |
| `uv` | objeto | paraxial | Sempre gravada. Como a textura é aplicada na face, veja [Projeção de textura](#texture-projection) |
| `props` | objeto de string para string | omitida | Atributos livres da face, como as chaves de mistura abaixo |
| `disp` | objeto | omitida | Transforma a face num [displacement](#displacement) |
| `colors` | array de `[r, g, b, a]` | omitida | [Pintura de vértices](../editor/texturing.md#vertex-paint), uma cor por índice |

O build do Godot pula um brush com menos de quatro vértices ou faces, já que nenhum sólido fechado tem menos.

Uma face também pode misturar dois materiais. As props de face `blend_material`, `blend_detile`, `blend_uv_scale` e `blend_detile_sharpen` configuram isso, e depois o alfa da cor de vértice esmaece de `material` para `blend_material` ao longo da face.

## Projeção de textura { #texture-projection }

`uv` é uma projeção no estilo Valve 220 com eixos explícitos: duas direções do mundo dizem para que lado a textura corre pela face, e um deslocamento e uma escala a posicionam.

| Chave | Tipo | Significado |
| --- | --- | --- |
| `u_axis`, `v_axis` | `[x, y, z]` | Direção no mundo do u e do v da textura |
| `offset` | `[u, v]` | Deslocamento em texels |
| `scale` | `[u, v]` | Unidades do mapa por texel |
| `rotation` | número | Opcional e só informativo, os eixos já contêm a rotação |

Um ponto `p` cai no texel `dot(p, u_axis) / scale.x + offset.x`, e o mesmo vale para o v. Para o UV final, divida isso pelo tamanho de textura. É o `texture_size` do material quando ele tem um, veja [Materiais](../godot/materials.md), senão o tamanho em pixels da imagem.

Um `uv` ausente significa `u_axis = [1, 0, 0]` e `v_axis = [0, 0, 1]` na escala 1. Quando `uv` está presente, toda chave exceto `rotation` é obrigatória.

## Configurações de textura { #texture-settings }

O objeto `textures` de nível superior guarda configurações compartilhadas por toda face com um material, indexadas pelo nome do material. Só as configurações que diferem do padrão são gravadas, e um material sem nenhuma fica de fora.

| Chave | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `bake` | bool | `true` | As faces ganham um lugar no lightmap. Desligado, o Godot dá a elas a luz média pré-calculada |
| `casts` | bool | `true` | As faces bloqueiam a luz no bake |
| `texel_scale` | número | `1` | Multiplica o tamanho do texel do lightmap nessas faces, de 0,125 a 16 |
| `size` | `[w, h]` | nenhum | Unidades do mapa que uma repetição cobre, usado no lugar do tamanho de textura acima |
| `projection` | string | `"face"` | `"world"` ignora o `uv` de cada face do brush e usa a projeção redefinida da sua normal, deslocamento 0 e escala 1 |

`world` vale só para faces de brush, as faces de malha mantêm a projeção delas.

## Displacement { #displacement }

Uma face quadrada pode levar um displacement no estilo Hammer, uma grade de vértices que são empurrados, cada um, ao longo da normal da face.

| Chave | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `power` | inteiro | obrigatória | De 1 a 4. A grade tem `2^power + 1` vértices por lado |
| `heights` | array de números | obrigatória | Quanto cada vértice da grade é empurrado ao longo da normal |
| `alphas` | array de números | omitida | Peso de mistura por vértice da grade, 0 mostra o material da face |

Os dois arrays são row major com `(2^power + 1)^2` entradas. As linhas vão do primeiro canto da face em direção ao quarto, as colunas do primeiro canto em direção ao segundo.

## Malha { #mesh }

Uma malha poligonal editável. Ao contrário de um brush, ela pode ser côncava, aberta ou ter faces que não são planas.

| Chave | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `vertices` | array de `[x, y, z]` | obrigatória | Cantos em unidades do mapa |
| `faces` | array de faces | obrigatória | As mesmas chaves das faces de brush, mais `uvs` |
| `smooth_angle` | número | `0`, omitida | Faces que se encontram com menos que esta quantidade de graus compartilham normais e são sombreadas suavemente. 0 é sombreamento chapado |
| `decal` | bool | `false`, omitida | Marca a malha como um [decal](../editor/decals.md) |

A suavização compara cada canto de face com cada face ao redor do vértice. Uma malha que precisa de mais de 64 comparações por canto, ou de 262.144 no total, é sombreada chapada em vez disso. Só um vértice compartilhado por centenas de faces, como a ponta de um cone de mais de uns 650 lados, chega lá.

O `uvs` de uma face de malha é um array de `[u, v]`, um por índice, em larguras de textura. Quando existe, ele substitui o `uv`, e uma quantidade que não bate com `indices` faz o editor recusar o mapa.
