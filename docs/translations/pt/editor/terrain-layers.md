---
source: 7732f7c6b2b4601df9d9b8913237f6914fb6ca69
---

# Camadas de terreno { #terrain-layers }

Um terreno tem de uma a quatro camadas, cada uma um material mais algumas configurações de como ele se repete. Selecione o terreno e você as encontra no Inspetor. As camadas dividem o chão entre si, então pintar areia sobre a grama tira peso da grama.

![O Inspetor de um terreno com quatro camadas, cada uma com seu material, ladrilho, desladrilhar e nitidez](../assets/terrain-layers/layers-inspector.png)

| Para | Faça isto |
| --- | --- |
| Adicionar uma camada | **+ camada (material atual)** embaixo das camadas, que usa o material em que você clicou por último no painel Materiais. Aparece enquanto o terreno tem menos de quatro |
| Remover a última camada | **- última camada** |
| Trocar o material de uma camada | Digite o nome dele no campo da camada, ou arraste um material do painel Materiais para o terreno. A linha no fim do Inspetor diz para qual camada um arraste vai, e com a ferramenta Mistura ativa é a camada que a ferramenta pinta |

**A camada 0 é o chão base.** O chão sem pintura mostra a camada 0 e apagar volta a pintá-lo com ela, então ponha ali o seu chão mais comum, geralmente grama.

Um terreno novo recebe as camadas nomeadas na janela *Criar terreno*, quatro por padrão. Soltar um material para uma camada que o terreno ainda não tem a adiciona como a próxima camada.

## Ladrilho e desladrilhamento { #tile-and-de-tiling }

Qualquer textura repetida sobre uma área grande mostra um padrão de grade de longe. Estas configurações trocam nitidez de perto pelo disfarce desse padrão.

| Configuração | Significado |
| --- | --- |
| ladrilho | Quantas unidades de mapa uma repetição da textura cobre. De 128 a 384 funciona bem para o chão onde se anda |
| desladrilhar | De 0 a 1. Gira e desloca cada repetição aleatoriamente para a grade parar de aparecer |
| nitidez | De 0 a 1, aparece quando desladrilhar passa de 0. Com 0 as repetições deslocadas se misturam por igual, o que fica macio. Com 1 elas se misturam só ao longo das emendas, o que mantém a textura nítida |

Um ladrilho pequeno é nítido de perto mas repete de forma visível de longe, um grande desfoca aos seus pés. Desladrilhar esconde a repetição sem um ladrilho maior, mas custa leituras extras de textura, então deixe em 0 onde a repetição não aparece.

![Ladrilho 256 com desladrilhar 0 à esquerda e 0,7 à direita](../assets/terrain-painting/detile-compare.jpg)

> [!NOTE]
> O `metadata/texture_size` do material não vale para terreno, só o **ladrilho** vale.

## Como fica no Godot { #how-it-looks-in-godot }

Cada camada é projetada de cima e dos lados, então os penhascos mostram pedra em vez de um borrão, e combina com o que você vê no editor. Cada camada usa a textura de cor, o mapa normal, a rugosidade e a emissão do material dela. A oclusão de ambiente é usada quando está na textura de rugosidade, como numa textura ORM, ou quando a camada não tem textura de rugosidade.

![Os penhascos pintados da ilha do mar no Godot](../assets/terrain-painting/godot-cliffs.jpg)
