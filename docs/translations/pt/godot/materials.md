---
source: 6f3b7a09923ac7b7e6e7e7d49ae11666c6d47d02
---

# Materiais { #materials }

Toda face aponta para um material pelo caminho dele dentro do *Base Texture Dir* das configurações do mapa, sem a extensão, por
exemplo `showcase/planks`.

Quando o Godot constrói o mapa, ele procura `showcase/planks.tres` nessa pasta, ou no *Base Material Dir* se você
definiu um, e usa o arquivo como está. Sem `.tres`? Então o addon faz um material simples a partir da imagem. O TrenchDot mostra a prévia
do mesmo `.tres`, então os dois lados ficam parecidos.

Você só precisa de um `.tres` quando uma imagem simples não basta, por exemplo para mudar a escala, adicionar brilho ou deixar
o material transparente. Um `.tres` só com um `albedo_color` e sem textura também funciona, o editor o desenha nessa cor.

Texturas e materiais novos aparecem na aba Materiais em poucos segundos, sem precisar recarregar.

## Tamanho de textura { #texture-size }

Por padrão um pixel de textura cobre uma unidade do mapa, então uma textura de 64 pixels se repete a cada dois metros. Ótimo para pixel art,
mas uma foto de 1024 pixels de uma parede de dois metros se esticaria por 32 metros.

Para resolver isso, diga ao material quantas unidades do mapa uma repetição cobre:

```
[resource]
albedo_texture = ExtResource("1_albedo")
metadata/texture_size = Vector2(64, 64)
```

(Você também pode adicioná-lo no inspetor do Godot em *Metadata*, como um `Vector2` ou um número só.)

Passe o mouse sobre o material na aba Materiais para conferir, a dica mostra o tamanho da imagem e quantas unidades uma repetição
cobre.

![A aba Materiais com a dica de night/shop_sign: 1024 x 256 px, 256 x 64 unidades por repetição, emissivo x3](../assets/materials/tooltip-texture-size.png)

Agora você pode trocar por uma imagem mais nítida sem mudar o mapa. As camadas de terreno ignoram isso, a configuração `tile` delas já
define a repetição.

## Materiais que brilham { #glowing-materials }

Um material brilha quando o arquivo dele define `emission_enabled`. Com uma cor de emissão preta, a textura de emissão brilha exatamente
como foi pintada:

```
[resource]
albedo_texture = ExtResource("1_albedo")
emission_enabled = true
emission = Color(0, 0, 0, 1)
emission_energy_multiplier = 2.5
emission_texture = ExtResource("2_emission")
```

Sem arquivo de material? Ponha uma imagem com o nome do albedo mais `_emission`, ao lado dele, `night/sign_emission.png` ao lado
de `night/sign.png`, e ela vira o mapa de brilho.

Materiais que brilham ganham uma etiqueta amarela *brilho* na aba Materiais:

![Materiais da pasta night na aba Materiais, onde apartment_lit, neon_red e shop_sign têm uma etiqueta amarela de brilho](../assets/materials/glow-tags.png)

Aperte F3 para a Prévia iluminada e vê-los brilhar na vista 3D. O bloom em volta deles só aparece no Godot, veja
[Iluminação noturna](night-lighting.md).

## Transparência { #transparency }

Defina `transparency` no arquivo do material quando o canal alpha da textura deve deixar as coisas aparecerem por trás. Cada modo
troca qualidade por custo de um jeito diferente:

| `transparency` | Como funciona | Use para | Cuidado |
| --- | --- | --- | --- |
| `1` blend | Mistura a superfície com o que está atrás dela | Vidro, manchas suaves, fumaça | Não projeta sombras |
| `2` scissor | Os pixels ou aparecem inteiros ou são cortados | Recortes duros de perto, como grades e folhas | Partes finas somem à distância |
| `3` hash | Faz dithering dos pixels, quanto menor o alpha menos aparecem | Recortes finos vistos de longe, como cercas de arame | Bordas suaves viram chuvisco |

A dica na aba Materiais diz qual modo um material usa:

![A aba Materiais com a dica de night/chainlink: alpha-hash, mapa normal, face dupla](../assets/materials/tooltip-alpha-hash.png)
