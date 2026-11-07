---
source: 7757565706801cf3511cdef72eaa79c5eb0dc3d0
---

# Pintura de terreno { #terrain-painting }

A ferramenta **Mistura** (Shift+G) pinta as [camadas](terrain-layers.md) de um terreno, por exemplo areia ao longo de uma praia ou pedra num penhasco. Como a Esculpir, ela age nos terrenos selecionados, ou em todos eles quando nada está selecionado.

## A ferramenta Mistura { #the-blend-tool }

1. Escolha a ferramenta

    Aperte Shift+G, ou use *Terreno > Mistura > Ferramenta de mistura*. As opções dela aparecem na barra de opções da ferramenta no canto superior esquerdo.

    ![As opções da ferramenta Mistura no modo inclinação: modo, atenuação, camada, raio, força e a faixa de inclinação](../assets/terrain-painting/blend-toolbar.png)

2. Escolha um modo

    O primeiro menu da barra decide o que o brush pinta, os modos estão listados abaixo.

    ![O menu de modos da ferramenta Mistura aberto com inclinação sob o mouse](../assets/terrain-painting/blend-modes.png)

3. Escolha a camada e pinte

    Defina **camada** (0 a 3) com a camada que você quer, depois arraste sobre o terreno. Várias passadas leves dão bordas mais macias do que uma passada forte.

    ![Pintando um caminho de terra subindo a ilha com a camada 2 no modo pintar](../assets/terrain-painting/blend-paint-path.gif)

| Modo | O que pinta |
| --- | --- |
| pintar | Adiciona a camada escolhida sob o brush |
| apagar | Pinta de volta em direção à camada 0, o chão base |
| suavizar | Amacia as bordas entre camadas |
| nitidez | Deixa a camada mais forte vencer, para bordas nítidas |
| ruído | Só em manchas em forma de nuvem do **tamanho** dado, para musgo, grama seca ou cascalho |
| inclinação | Só onde o chão está entre dois ângulos. `30` a `90` põe pedra em tudo o que é mais íngreme que 30° |
| altura | Só entre duas alturas do mundo, para areia na linha d'água ou neve acima da linha das árvores |

Inclinação e altura tornam seguros os brushes enormes, um brush do tamanho de uma colina no modo inclinação só transforma os penhascos em pedra.

O segundo menu da barra é a atenuação, como a força se esvai do centro do brush até a borda.

| Atenuação | Forma |
| --- | --- |
| suavizar | Um esmaecer suave, a escolha de sempre |
| linear | Um esmaecer uniforme do centro até a borda |
| constante | O disco todo com a mesma força, para bordas duras |
| spray | Pontinhos aleatórios, para pedrinhas e flores |

Shift transforma pintar, ruído, inclinação e altura em apagar, e Ctrl suaviza. Ctrl+roda redimensiona o brush.

(Nota: **Definir material de mistura**, **repetição**, **desladrilhar**, **nitidez** e **Aplicar ladrilhamento**, mais à direita na barra, servem para misturar faces de brush, um terreno pega essas configurações das camadas dele.)

## Pintar auto { #auto-paint }

Pintar auto repinta um terreno a partir da forma dele, uma primeira passada rápida antes de pintar à mão.

1. Selecione o terreno

    Pintar auto só mexe nos terrenos selecionados.

2. Execute

    Vá em *Terreno > Mistura > Pintar camadas automaticamente*, ou clique em **Pintar auto** no Inspetor.

    ![O menu Terreno > Mistura com Pintar camadas automaticamente sob o mouse e Faixas a partir do nível do mar marcado em y 0](../assets/terrain-painting/menu-auto-paint.png)

| Camada | Vai em |
| --- | --- |
| 1 | Declives mais íngremes que cerca de 50° |
| 2 | Os 20% de cima da faixa de altura |
| 3 | Os 8% de baixo da faixa de altura |

A camada 0 cobre todo o resto. Um terreno com menos camadas recebe as faixas que faltam na última camada dele.

A faixa começa no ponto mais baixo, que numa ilha é o fundo do mar. Marque **Faixas a partir do nível do mar** para medir a partir de uma altura que você der, como a linha d'água em y 0 acima.

![A ilha do mar antes de Pintar auto, toda de grama](../assets/terrain-painting/auto-paint-before.jpg)

![A mesma ilha depois de Pintar auto: pedra nos penhascos, terra no pico e areia ao longo das praias](../assets/terrain-painting/auto-paint-after.jpg)

> [!WARNING]
> Pintar auto substitui toda a pintura, então esculpa primeiro, pinte automaticamente e pinte à mão por último.

## Quão estreito você consegue pintar { #how-narrow-you-can-paint }

A pintura vive nos vértices, então um traço precisa de um raio de cerca de duas células para uma faixa limpa. Abaixo de uma célula ele vira uma escada. Para uma trilha estreita, use células menores, ou coloque uma malha ou um [decal](decals.md) por cima.

![Traços com raio 40, 96 e 160 em células de 64 unidades, ao lado da grade de vértices que guarda a pintura](../assets/terrain-painting/brush-width-grid.jpg)
