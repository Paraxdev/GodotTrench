---
source: 5b022e12202fe97edce0105dd2916cbd30e66e36
---

# Terreno { #terrain }

Um terreno é uma grade grande de alturas que você esculpe como argila, com até quatro texturas pintadas por cima. Use-o para chão aberto, como colinas, praias e vales, onde moldar brushes à mão levaria uma eternidade.

Esta página cobre como criar um e dar forma a ele com a ferramenta Esculpir (G). Texturizá-lo está em [Camadas de terreno](terrain-layers.md) e [Pintura de terreno](terrain-painting.md).

## Criando um terreno { #creating-a-terrain }

1. Abra a janela

    Vá em *Terreno > Criar terreno…* na barra de menus.

    ![O menu Terreno aberto com Criar terreno… sob o mouse](../assets/terrain/menu-create-terrain.png)

2. Escolha um tamanho

    **Resolução** e **Tamanho da célula** decidem o tamanho que ele terá, e a linha **Tamanho** embaixo delas mostra o resultado em unidades e metros. O padrão de 129 x 129 vértices com células de 64 unidades dá 8192 unidades, 256 m. Veja [Escolhendo o tamanho da célula](#picking-the-cell-size) abaixo.

    ![A janela Criar terreno com os padrões: 129 x 129, células de 64 u, Colinas e os quatro campos de camada](../assets/terrain/create-terrain-dialog.png)

3. (Opcional) Escolha uma forma inicial

    **Forma** é o chão de onde você parte. O padrão é Colinas, as outras são Plano, Montanha, Ilha, Vale e Cristas. **Importar PNG…** lê um mapa de altura em tons de cinza no lugar, onde pixels mais claros são mais altos. As duas opções só funcionam aqui, na criação.

4. (Opcional) Escolha as camadas

    **Camada base**, **Camada de declive**, **Camada de pico** e **Camada baixa** são as quatro texturas do terreno, e **Usar atual** coloca o material em que você clicou por último. Com **Pintar camadas por declive e altura** marcado elas são pintadas na hora, veja [Pintar auto](terrain-painting.md#auto-paint).

5. Clique em **Criar** na parte de baixo

    O terreno fica centralizado no cursor 3D, ou no **Centro** que você digita com **Cursor 3D** desmarcado, e essa altura vira o zero do terreno. Você deve ver o novo terreno selecionado, com a ferramenta Esculpir já ativa.

## Escolhendo o tamanho da célula { #picking-the-cell-size }

| Configuração | Significado |
| --- | --- |
| Resolução | Quantos vértices correm ao longo de cada lado, de 33 a 1025. Mais vértices guardam mais detalhe, mas custam mais para desenhar e esculpir |
| Tamanho da célula | A distância em unidades de mapa entre dois vértices vizinhos |

257 vértices com células de 64 unidades, por exemplo, dão 16384 unidades, 512 m.

Nada fica entre dois vértices, então o tamanho da célula é o detalhe mais fino que o terreno consegue guardar. Uma cordilheira pede células grandes, um jardim pede pequenas.

Escolheu errado? Selecione o terreno e abra **Reamostrar** no Inspetor. Ele refaz a grade com 65, 129, 257 ou 513 vértices ao longo do lado maior e mantém as alturas, a pintura e os buracos. O terreno mantém o tamanho, então mais vértices significam células menores.

![O Inspetor do terreno com o menu Reamostrar aberto e 257 vértices sob o mouse](../assets/terrain/inspector-resample.png)

**Células por chunk** (32 por padrão) divide o terreno em pedaços quadrados para o desenho e a colisão. Chunks menores se refazem mais rápido durante a escultura, maiores significam menos nós no Godot.

## Esculpindo { #sculpting }

Aperte G, escolha um modo na barra de opções da ferramenta no canto superior esquerdo e arraste sobre o terreno. Ela age nos terrenos selecionados, ou em todos eles quando nada está selecionado.

![O menu de modos da ferramenta Esculpir aberto na barra de opções da ferramenta, ao lado dos campos de raio e força](../assets/terrain/sculpt-modes.png)

Mantenha o botão do mouse apertado e o chão continua subindo sob o brush.

![Segurando Elevar com um brush de 900 unidades, uma colina se forma na ilha](../assets/terrain/sculpt-raise.gif)

| Modo | O que faz |
| --- | --- |
| Elevar, Abaixar | Sobe a altura ou a escava enquanto você segura o botão. Um clique faz um calombo pequeno |
| Suavizar | Relaxa cada vértice em direção aos vizinhos, para amaciar calombos e cristas agudas |
| Nivelar | Puxa o chão para a altura em que o traço começou, para construir plataformas, estradas e platôs |
| Ruído | Acrescenta pequenos calombos aleatórios, para o chão plano parecer menos artificial |
| Terraço | Prende as alturas em degraus de **passo** unidades, para arrozais ou penhascos em degraus. O campo aparece ao lado do modo |
| Pintar alfa, Apagar alfa, Pintar camada | Pintam em vez de moldar. Os modos alfa são pensados para displacements, Pintar camada pinta a camada de terreno definida ao lado do modo. Num terreno, a [ferramenta Mistura](terrain-painting.md) faz isso com mais controle |
| Buraco, Tapar buraco | Recorta células inteiras ou as preenche de volta, colisão incluída. Use para entradas de caverna ou de bunker |

Ctrl+roda redimensiona o brush, Shift troca elevar e abaixar, e segurar Ctrl suaviza seja qual for o modo.

**força** define a rapidez com que o brush trabalha. Elevar e Abaixar movem o centro do brush 8 unidades por segundo por ponto de força, seja qual for o raio, então o padrão 4 constrói 32 unidades por segundo. Para colinas e ilhas aumente a força em vez de segurar por um minuto.

Mantenha o raio acima do tamanho da célula do terreno, um brush menor só empurra o único vértice sob ele (a barra de status avisa quando isso acontece).

Para recomeçar, *Terreno > Esculpir > Nivelar*, ou **Nivelar** no Inspetor, volta o terreno selecionado inteiro à altura zero.

> [!TIP]
> Dê forma com um brush grande e suavize depois de cada passada grosseira. Brushes pequenos deixam calombos.

## No Godot { #in-godot }

Todo terreno vira um nó `TrenchDotTerrain`, um `StaticBody3D` (o nó do Godot para chão sólido que nunca se move) cuja colisão corresponde ao que ele desenha. Para o código de gameplay, `height_at(x, z)` devolve a altura do chão numa posição local, útil para pôr coisas no chão durante a execução.

<div class="td-mcp" markdown>

## MCP { #mcp }

A op `sculpt` da ferramenta `terrain_edit` aplica uma única pincelada, e a força dela é o quanto essa pincelada move o centro, de novo seja qual for o raio. Segurar um traço por um segundo com força 4 equivale a uma pincelada de força 32.

</div>
