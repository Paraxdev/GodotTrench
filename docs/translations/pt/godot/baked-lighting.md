---
source: 5d04cb3d9a7e005f4a371160c23d337e8ee707a7
---

# Iluminação pré-calculada no Godot { #baked-lighting-in-godot }

Quando você [pré-calcula um mapa no editor](../editor/light-baking.md), o lightmap é salvo dentro do `.tdm`. O Godot
não pré-calcula nada por conta própria, cada construção só transforma o bake guardado no nó de lightmap do próprio Godot.

## O que a construção adiciona { #what-the-build-adds }

| Parte | O que ela recebe |
| --- | --- |
| Um `LightmapGI` chamado `baked_lighting` | Um filho do `FuncGodotMap`, que guarda o lightmap, a máscara de sombra do sol e as sondas de luz |
| Malhas pré-calculadas | As coordenadas de lightmap delas como UV2 e um modo de GI estático, chunks de streaming incluídos |
| Luzes e o sol | O modo de bake do Godot vindo de `bake_mode` ou `sun_bake_mode`, veja [Keys de luz](../editor/light-baking.md#light-keys) |

Com o sol em `bounce`, o sol continua em tempo real e a máscara de sombra pré-calculada escurece o que fica além da distância da sombra dele.

Faces que você mudou depois do bake voltam para a luz média do mapa, então parecem plausíveis mas chapadas até você pré-calcular
de novo.

## Sondas de luz { #light-probes }

Objetos em movimento, como o jogador, props e portas, pegam a luz das sondas de luz ao redor deles, então recebem
a luz rebatida pré-calculada e se misturam com o mapa.

O bake coloca sondas numa grade de 2 metros sobre a caixa em volta da geometria pré-calculada e descarta as que ficam dentro de sólidos. Onde
a grade é grossa demais, como um corredor estreito ou uma escada, adicione uma entidade `light_probe` para ter mais uma. Um mapa recebe cerca de
1000 sondas no máximo, as colocadas à mão incluídas, então num mapa maior a grade se espalha.

## Entidades de sonda { #probe-entities }

| Entidade | Tipo | Nó do Godot | Use para |
| --- | --- | --- | --- |
| `light_probe` | Ponto | `LightmapProbe` | Uma sonda de luz extra para o bake, veja acima |
| `env_reflection_probe` | Volume de brush | `ReflectionProbe` do tamanho do brush | Reflexos que combinam com uma sala em pisos brilhantes e metal |
| `env_voxel_gi` | Volume de brush | `VoxelGI` do tamanho do brush | Luz rebatida em tempo real que acompanha luzes e objetos em movimento |

Você acha as três na aba Entidades. Arraste `light_probe` para uma vista. Para os dois volumes, desenhe uma caixa em volta da
sala, depois clique com o botão direito no cartão e escolha **Converter os brushes selecionados nela**.

![A aba Entidades filtrada por env_, com o cartão env_voxel_gi clicado com o botão direito e Converter os brushes selecionados nela em destaque](../assets/baked-lighting/entities-turn-brushes-into.png)

O brush só dá a caixa, a construção o remove. As keys das três estão listadas em
[Props, luzes e efeitos](../gameplay/entities/effects.md#light_probe-env_reflection_probe-and-env_voxel_gi).

Um `env_voxel_gi` é pré-calculado no editor do Godot toda vez que o mapa é construído. Num volume grande, defina `bake_on_build` como 0 e
pré-calcule à mão com o botão **Pré-calcular** do nó `VoxelGI`. Mapas construídos enquanto o jogo roda pulam esse bake.

## Usando outro lightmapper { #using-another-lightmapper }

Quer pré-calcular com o próprio `LightmapGI` do Godot? Desmarque **Use Baked Lighting** no `FuncGodotMap` para deixar o
bake guardado de fora. A construção então não adiciona o nó `baked_lighting` e deixa os modos de bake das luzes em paz. Marque *Unwrap
UV2* em *Build Flags* para que as malhas recebam o UV2 de que um bake do Godot precisa.
