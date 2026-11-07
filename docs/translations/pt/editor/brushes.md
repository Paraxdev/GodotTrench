---
source: 6e5c9bc3787f689328791dca9df024446698b311
---

# Brushes e CSG { #brushes-and-csg }

Os brushes são o que você usa para construir a maior parte de um nível. Desenham-se rápido, se encaixam direitinho na grade e viram malhas com colisão correspondente no Godot. Tudo aqui usa a ferramenta **Selecionar** (Q), a não ser que uma seção cite outra ferramenta.

## Desenhar e mover { #draw-and-move }

Arraste em qualquer lugar fora da seleção, em qualquer vista, e uma caixa segue o mouse. As vistas 2D escrevem o tamanho dela ao lado. Na vista 3D ela começa na superfície sob o cursor e tem um passo de grade de espessura. Numa vista 2D ela pega a profundidade do último brush que você desenhou, 128 unidades para o primeiríssimo.

![As quatro vistas enquanto uma caixa é desenhada em 3D, a nova caixa tem 144 por 144 unidades e um passo de grade, 16 unidades, de espessura](../assets/brushes/draw-box-3d.png)

Com um brush selecionado, você pode movê-lo por aí:

| Entrada | Resultado |
| --- | --- |
| Arrastar a seleção | Move, Shift trava no eixo principal |
| Ctrl+arrastar a seleção | Move uma cópia, o original fica |
| Alt+arrastar a seleção (3D) | Move na vertical. Começando fora da seleção, Alt+arrastar orbita a câmera |
| Setas, Page Up, Page Down | Empurra um passo de grade |
| `[` e `]` | Divide ao meio e dobra a grade |

Em 3D a seleção também ganha um gizmo de mover, girar e escalar. A rotação encaixa em 15°, ou em 1° com Shift. Se ele atrapalhar, desligue em *Exibir > Gizmo de transformação*.

## Posição e tamanho exatos { #exact-position-and-size }

Às vezes você quer números exatos, como um cortador de porta com exatamente a profundidade da parede. Selecione o brush e digite-os no **Inspetor**, à direita.

![O Inspetor com um brush selecionado, mostrando os campos de posição e tamanho](../assets/brushes/inspector-position-size.png)

A posição é o canto mais baixo, os menores X, Y e Z, que é o canto que costuma ficar na grade. Um tamanho novo faz a seleção crescer para longe desse canto, então a posição não sai do lugar. Arrastar um campo avança pela grade enquanto o encaixe está ligado, um valor digitado é mantido exatamente. Os mesmos campos aparecem para malhas, uma entidade brush e uma seleção de vários objetos.

## Remodelar { #reshape }

Você pode remodelar um brush selecionado sem sair da ferramenta Selecionar:

| Entrada | Resultado |
| --- | --- |
| Shift+arrastar uma face selecionada (3D) | Empurra ou puxa essa face |
| Ctrl+Shift+arrastar uma face selecionada (3D) | Extrai um brush novo a partir dela |
| Arrastar uma aresta da seleção (2D) | Redimensiona. A aresta acende em amarelo e o cursor muda onde um arrasto redimensiona |

![Shift+arrastar a face lateral de uma caixa selecionada na vista 3D, a caixa estica em passos de grade](../assets/brushes/push-face.gif)

Para rampas, declives e paredes inclinadas, aperte C para a ferramenta **Clip**:

1. Clique em dois ou três pontos ao longo do corte. Eles encaixam na grade, e em 3D caem na superfície em que você clica.
2. Aperte Tab para escolher o que manter, a frente, o verso ou as duas metades. O texto no pé da vista diz qual está ativa.
3. Aperte Enter para cortar. Backspace remove o último ponto se você clicou errado.

![A ferramenta Clip com dois pontos postos na vista Frente atravessando o canto superior de uma caixa, a linha de corte também aparece na vista 3D](../assets/brushes/clip-points.png)

Para arrastar cantos isolados, aperte V para a ferramenta **Vértice**, ou Tab com brushes selecionados. Ela mostra alças nos cantos e pequenos pontos azuis no meio das arestas e no centro das faces. Clique em um ponto para selecioná-lo, ou arraste-o para longe do brush para acrescentar um canto ali. Delete remove os vértices selecionados.

![A ferramenta Vértice numa caixa cortada, alças amarelas nos cantos e pontos azuis nas arestas e faces](../assets/brushes/vertex-handles.png)

> [!NOTE]
> Um brush não pode ter reentrâncias, então um movimento que criaria uma é recusado. Você vê um fantasma vermelho onde ele cairia e uma linha na barra de status. Puxe o ponto para fora, ou converta o brush em uma [malha](meshes.md) para ter reentrâncias e saliências.

## Girar, escalar e medir { #rotate-scale-and-measure }

| Ferramenta | Entrada | Resultado |
| --- | --- | --- |
| **Girar** (R) | Arrastar um anel em volta da seleção | Gira em passos de 15°, com Shift em passos de 1°. Uma vista 2D tem um anel, em volta do eixo ao longo do qual ela olha |
| **Escalar** (T) | Arrastar uma alça nos limites da seleção | Redimensiona na grade, com Alt dos dois lados de uma vez para o centro ficar parado. A vista 3D tem uma alça em cada lado |
| **Medir** (M) | Arrastar, ou clicar em dois pontos | Mostra a distância em unidades e metros, e no pé da vista suas partes X, Y, Z e horizontal |

![A ferramenta Medir na vista 3D, uma linha rotulada 230.76 u (7.21 m) e as partes X, Y e Z no pé](../assets/brushes/measure.png)

## Gerador de formas { #shape-generator }

Para cilindros, arcos, escadas, telhados e outras formas que dariam muito trabalho com brushes desenhados à mão, use o Gerador de formas.

1. Desenhe uma caixa onde a forma deve ficar, e deixe-a selecionada.
2. Abra *Brush > Gerador de formas…* (Ctrl+Shift+B).
3. Escolha a **Forma** e suas opções. A janela diz os limites que ela preenche, e que substitui a seleção.
4. Clique em **Criar**.

![A janela do Gerador de formas em Cilindro com 16 lados e o eixo Y, preenchendo uma seleção de 128 x 192 x 128](../assets/brushes/shape-generator.png)

![A caixa selecionada à esquerda, o cilindro de 16 lados que a substituiu à direita](../assets/brushes/shape-generator-box-to-cylinder.png)

Cilindros e cones ficam em pé por padrão. Escolha **X** ou **Z** em **Eixo** para deitar um dentro da mesma caixa, para canos, troncos ou uma viga, em vez de girá-lo depois. O toro, a parede em arco, o telhado de duas águas, a torre pontiaguda e a grade sempre são feitos como [malha](meshes.md). Cilindros, cones e esferas também podem ser uma com **Criar como malha editável**, e então **Sombreamento suave** os arredonda. A forma **Texto** constrói letras em bloco com brushes, para placas e letreiros no chão.

## CSG { #csg }

As operações de CSG ficam em *Brush > CSG*, que também guarda as duas configurações delas.

![O menu Brush com o submenu CSG aberto, listando Subtrair, Fusão convexa, Intersectar e Esvaziar com seus atalhos, a caixa Faces recortadas mantêm o material do alvo e o campo Espessura da parede](../assets/brushes/menu-brush-csg.png)

Todas as quatro também têm um botão na barra de ferramentas, à direita dos botões de encaixe e de bloqueio de UV.

![Os quatro botões de CSG na barra de ferramentas, Fusão convexa sob o mouse com sua dica](../assets/brushes/toolbar-csg.png)

| Operação | Tecla | Resultado |
| --- | --- | --- |
| Subtrair | Ctrl+K | Recorta a seleção de todo brush que ela toca, depois apaga a seleção. Uma seleção que não toca nenhum outro brush é mantida e a barra de status avisa |
| Esvaziar | Ctrl+Shift+K | Transforma um brush em paredes em volta do seu antigo volume, com 16 unidades de espessura, a não ser que você mude *Espessura da parede* no mesmo menu. Um brush fino demais para duas paredes é deixado em paz |
| Fusão convexa | Ctrl+M | Funde a seleção em um brush convexo que a envolve |
| Intersectar | Ctrl+L | Mantém só onde os brushes selecionados se sobrepõem |

Para abrir uma porta ou janela, ponha um brush onde o buraco vai ficar e use Subtrair, [Primeiros passos](../getting-started.md) te leva por um. As faces que um corte abre ficam com o material do cortador, a não ser que você marque *Faces recortadas mantêm o material do alvo* no mesmo menu.

## Displacements { #displacements }

Para um terreno irregular dentro de um nível de brushes, como uma rampa de entulho, você pode transformar uma face em um displacement e esculpi-la.

1. Shift+clique em uma face de quatro lados, ou selecione o brush para usar a face de cima dele.
2. Abra *Brush > Displacement* e escolha uma das entradas **Criar**. Uma potência maior dá uma grade mais fina.
3. A ferramenta **Esculpir** (G) liga sozinha, arraste sobre a face para dar forma a ela.

![O menu Brush com o submenu Displacement aberto, Criar, potência 2 (5x5), Criar, potência 3 (9x9) e Criar, potência 4 (17x17), depois Costurar displacements e Remover displacement](../assets/brushes/menu-brush-displacement.png)

Para áreas externas maiores, um [terreno](terrain.md) funciona melhor.
