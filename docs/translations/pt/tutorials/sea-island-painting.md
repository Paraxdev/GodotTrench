---
source: 6da371c3430389f428469b408b32808d1d2b0af6
---

# Ilha no mar 3: pintar { #sea-island-3-painting }

Mude para a ferramenta Mistura com **Shift+G**, ou o botão Mistura logo ao lado de Esculpir na barra de ferramentas. Como Esculpir, ela
age sobre o terreno selecionado ou sobre tudo quando nada está selecionado. Os ajustes dela ficam na barra de opções da ferramenta, no
canto superior esquerdo.

![A barra de ferramentas com o botão Ferramenta de mistura sob o mouse e a dica: arraste para pintar misturas nas camadas de terreno, displacements e faces com um material de mistura](../assets/sea-island/toolbar-blend.png)

Os modos **inclinação** e **altura** só pintam onde o chão está dentro de uma faixa de inclinação ou de altura. Um raio de
12000 cobre a ilha inteira, então um clique nesses modos pinta de uma vez todas as costas ou penhascos.

## Areia e pedra { #sand-and-rock }

| Ajuste | Areia | Pedra |
| --- | --- | --- |
| Modo | altura, de `-3000` a `40` | inclinação, de `32` a `90` |
| Atenuação | constante, força total em todo o pincel | constante |
| Camada | 3 | 1 |
| Raio, força | 12000, 1 | 12000, 1 |

Defina isso e clique uma vez no meio da ilha para cada um. Você deve ver tudo abaixo de 40 virar areia, e
toda encosta íngreme virar pedra:

![Modo altura: tudo abaixo de 40 unidades vira areia](../assets/sea-island/sand-height.jpg)

![Modo inclinação: tudo mais íngreme que 32° vira pedra](../assets/sea-island/rock-slope.jpg)

Para alargar a praia, pinte a camada 3 ao longo da costa sul no modo **pintar**, atenuação suavizar, raio 700, força 0,7.
A atenuação suave esmaece em direção à borda do pincel, então a borda da areia continua macia.

## Trilhas { #paths }

Modo **pintar**, atenuação suavizar, camada 2, raio 140, força 0,8. Arraste:

1. Do alto da praia até o morro principal, em curvas longas.
2. Do topo do morro principal ao longo da crista até o alto do penhasco.
3. Da ponta leste da praia até o segundo morro.

![Pintando a trilha morro acima](../assets/sea-island/paths.jpg)

![A trilha de perto](../assets/sea-island/path-close.jpg)

## Quebre a repetição { #break-up-the-repeat }

De longe, uma textura que se repete aparece como uma grade. Selecione o terreno no Outliner (um clique numa vista pintaria
com a ferramenta Mistura), depois defina **desladrilhar** e **nitidez** em cada linha de Camada do Inspetor. A nitidez mantém a
textura desladrilhada definida em vez de borrada.

| Camada | Desladrilhar | Nitidez |
| --- | --- | --- |
| 0 grama | 0,5 | 0,5 |
| 1 penhasco | 0,7 | 0,4 |
| 2 terra | 0,5 | 0,5 |
| 3 areia | 0,4 | 0,5 |

![O Inspetor do terreno com as quatro linhas de Camada, cada uma com material, ladrilho, desladrilhar e nitidez](../assets/sea-island/inspector-terrain-layers.png)

Próximo: [Ilha no mar 4: scatter](sea-island-scatter.md).
