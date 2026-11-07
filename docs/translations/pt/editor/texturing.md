---
source: fb9984bb419921807428d1c050d90fa918ed7b18
---

# Texturização { #texturing }

## Aplicar materiais { #applying-materials }

Suas texturas ficam no painel **Materiais**, embaixo da janela, sob as vistas. A caixa de busca no canto superior esquerdo as filtra pelo nome, **Usados no mapa** e **Favoritos** reduzem a lista, e o controle deslizante define o tamanho das miniaturas.

![O painel Materiais com a caixa de busca, a lista de pastas, Usados no mapa, Favoritos e o controle de tamanho, e abaixo deles as miniaturas de material com dev/blue sob o mouse mostrando seu nome e tamanho](../assets/texturing/materials-panel.png)

| Faça isto | Resultado |
| --- | --- |
| Clicar em uma miniatura | Aplica à seleção, e a usa para brushes novos |
| Arrastar uma miniatura para uma face | Aplica a essa face |
| Shift ao soltar | Aplica ao brush ou malha inteiro |
| Alt ao soltar | Posiciona como [decal](decals.md) |

Enquanto você arrasta, um cartãozinho junto ao cursor lembra os modificadores, e a face muda assim que você solta.

![Arrastando withered/mossy_plaster para a frente de uma parede de tijolos, depois a mesma parede com o reboco só na face da frente](../assets/texturing/drop-on-face.png)

Para texturizar uma face sem arrastar, use Shift+clique nela numa vista primeiro, depois clique na miniatura. Ctrl+Shift+clique adiciona ou remove mais faces.

Clique com o botão direito em uma miniatura para ver mais: adicioná-la aos favoritos, selecionar as faces que a usam, trocá-la pelo material atual no mapa inteiro, abrir o editor de hotspots, ou *Mostrar no gerenciador de arquivos* para abrir a pasta da textura com o arquivo selecionado, por exemplo para editá-lo.

![O menu do botão direito de uma miniatura com Aplicar à seleção, Adicionar aos favoritos, Selecionar faces que a usam, Substituir showcase/red_bricks por ele no mapa, Editor de hotspots, Copiar nome e Mostrar no gerenciador de arquivos](../assets/texturing/material-context-menu.png)

Soltar um material num terreno define uma camada de terreno em vez disso, veja [Camadas de terreno](terrain-layers.md).

O bloqueio de UV (Ctrl+Shift+U) vem ligado por padrão, então as texturas continuam presas a um brush enquanto você o move.

## Cores embutidas { #built-in-colors }

O painel Materiais sempre oferece `dev/grey`, `dev/dark`, `dev/orange`, `dev/blue` e `dev/green`, texturas de grade para esboçar um nível antes de ele ter arte. Codificar salas por cor com elas sobrevive à construção, já que o addon traz as mesmas grades e o Godot as usa quando o projeto não tem textura com esse nome. Uma textura sua em, por exemplo, `res://textures/dev/grey.png` substitui uma delas.

## Texturas de ferramenta { #tool-textures }

O editor desenha as texturas de ferramenta transparentes e listradas, e a construção no Godot não dá a elas malha visual. Toda textura nas pastas `special/`, `tools/` ou `td/` conta, e também qualquer textura com o nome de uma das ferramentas abaixo, como `common/caulk`.

![Um bloco dev/dark, uma parede roxa transparente special/clip na frente dele e uma caixa laranja listrada special/trigger](../assets/texturing/tool-textures.png)

| Textura | No Godot |
| --- | --- |
| `special/clip`, `playerclip`, `nodraw`, `caulk`, `trigger`, `sky` e qualquer outra ferramenta | Colide mas não desenha nada. Um trigger mantém a forma de trigger |
| `special/skip`, `origin`, `hint`, `hintskip`, `null`, `areaportal` | Não constrói nada. Um brush só com essas faces não colide |
| `special/occluder` | Não desenha nada e não esconde os vizinhos, mas soma ao occluder de uma entidade que constrói um, como `func_geo` |

As configurações do mapa nomeiam as texturas de clip, skip, origin e sky, `special/clip` e assim por diante por padrão. Um brush de origin só define a origem da sua entidade com a textura que as configurações do mapa nomeiam.

## A ferramenta Textura { #the-texture-tool }

O alinhamento de uma face é onde a textura fica nela: o deslocamento, a escala e a rotação. Aperte **Shift+T** para a ferramenta Textura, que o edita no lugar, como o modo de edição de face do Hammer. Clique em uma face para selecioná-la. Ela fica tingida de vermelho e mostra uma caixa azul em volta de uma repetição da textura, mais duas linhas curtas para as direções U e V. A face sob o cursor ganha um contorno amarelo.

![A ferramenta Textura com a face da frente de uma parede de tijolos selecionada, uma caixa azul marca uma repetição da textura](../assets/texturing/texture-tool-face.png)

| Entrada | Resultado |
| --- | --- |
| Clique, Ctrl+clique | O clique seleciona uma face, Ctrl+clique adiciona ou remove uma |
| Arrastar uma face selecionada | Desliza a textura, o pixel sob o cursor continua sob ele |
| Setas | Deslizam pelo tamanho da grade em pixels, com Shift por um pixel |
| Ctrl+roda | Escala por 2, com Shift em passos finos |
| Alt+roda | Gira em 15°, com Shift em 1° |
| Alt+clique | Pega o material e o alinhamento de uma face |
| Clique direito | Aplica o material atual |
| Ctrl+clique direito | Aplica e redefine o alinhamento |
| Shift+clique direito | Cola o material e o alinhamento pegos |
| Alt+clique direito | Aplica o material atual, continuando o alinhamento da face selecionada por cima da aresta |

Alt+clique direito é como uma textura, como um acabamento ou uma fileira de tijolos, contorna um canto sem emendas.

A barra de opções da ferramenta no alto tem os botões **Justificar**. **Esquerda**, **Direita**, **Topo**, **Base** e **Centro** empurram a textura para esse lado da face, **Ajustar** a estica sobre a face, **Ajustar L** e **Ajustar A** só na largura ou só na altura. Com **Tratar como um** marcado eles agem em todas as faces selecionadas como uma única área, então uma textura cobre todas elas. **Alinhar à vista** projeta a textura ao longo da câmera 3D.

![A barra de opções da ferramenta Textura com Justificar Esquerda, Direita, Topo, Base, Centro, Ajustar, Ajustar L, Ajustar A, Tratar como um, Alinhar à vista e Ajuste ao hotspot](../assets/texturing/texture-tool-options.png)

Se você prefere números, o **Inspetor** mostra a mesma face em **Alinhamento**, com campos de deslocamento, escala e rotação e botões para espelhar, virar e redefinir. **Ferramentas de textura** abaixo dele define a densidade de texels e copia o alinhamento de face para face.

![O Inspetor com uma face selecionada, a seção Alinhamento com deslocamento, escala e rotação, os botões Justificar e a seção Ferramentas de textura](../assets/texturing/inspector-alignment.png)

Para faces com UVs explícitos, e para malhas, use o [Editor de UV](uv-editor.md).

## Escala da textura { #texture-scale }

Por padrão um pixel de textura cobre uma unidade de mapa, então uma textura de 64 pixels se repete a cada dois metros. Isso serve para texturas de baixa resolução, mas uma foto de 1024 pixels se esticaria por 32 metros. Defina `metadata/texture_size` no material com o tamanho no mundo que uma repetição deve cobrir, veja [Tamanho de textura](../godot/materials.md#texture-size). A linha do Inspetor sob o nome do material diz o que a face recebeu, como "1024 x 1024 px, 64 x 64 unidades a repetição".

Para definir só para um mapa, ou para fazer uma textura correr sem emendas por todas as faces, use as [Config. de textura](light-baking.md#texture-settings) no Inspetor. O tamanho de repetição dela vence o do material.

## Hotspots { #hotspots }

Os hotspots deixam uma única textura de trim sheet servir a muitas faces de formas diferentes. Você marca os retângulos dela uma vez, e o TrenchDot escolhe o certo para cada face.

1. Clique com o botão direito na trim sheet no painel Materiais e escolha **Editor de hotspots**. *Textura > Editor de hotspots…*
    também o abre, para o material da face selecionada ou, na falta dela, o atual.

    ![O menu Textura com Editor de hotspots… sob o mouse, Ajuste ao hotspot (Alt+H) no fim](../assets/texturing/menu-texture.png)

2. Arraste sobre a textura para desenhar um retângulo, ou defina uma grade e clique em **Dividir em grade**. Arraste um retângulo para movê-lo, a alça do canto dele para redimensioná-lo, o clique direito o apaga.
3. Clique em **Salvar**. Os retângulos vão para `<texture>.hotspots.json` ao lado do arquivo da textura.
4. Selecione as faces e use *Textura > Ajuste ao hotspot* (Alt+H), ou **Aplicar à seleção** no editor.

![O Editor de hotspots em withered/concrete_panels dividido numa grade de 4 por 4, 16 retângulos, não salvo](../assets/texturing/hotspot-editor.png)

O Ajuste ao hotspot dá a cada face o retângulo cuja forma mais combina com ela, preferindo um que mantenha a textura perto da escala normal.

## Pintura de vértices { #vertex-paint }

A ferramenta **Pintar** (P) pinta cores de vértice nas faces dos brushes, para sujeira, tonalidade e iluminação falsa. Escolha a cor na barra de opções da ferramenta, junto ao raio e à força do pincel, depois arraste sobre as faces. Ela pinta os brushes selecionados, ou todos os brushes quando nada está selecionado. Ctrl+roda redimensiona o pincel.

![A barra de opções da ferramenta Pintar com a amostra de Cor, raio 48 u e força 4.0](../assets/texturing/paint-tool-options.png)
