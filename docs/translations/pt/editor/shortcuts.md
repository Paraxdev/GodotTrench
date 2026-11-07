---
source: c1ad5f7b4051d96e108a3133de3b683670efbf64
---

# Atalhos { #shortcuts }

Estas são as teclas padrão, a predefinição do TrenchBroom.

Quer outras teclas? *Ajuda > Atalhos de teclado…* lista todas as associações. Digite no filtro para achar uma, aperte **Definir** e depois o novo atalho. **Predefinição** no topo dessa janela, ou *Arquivo > Preferências > Predefinição de atalhos*, troca para Hammer ou Blender, veja [Outras predefinições](#other-presets).

![A janela Atalhos de teclado filtrada por csg, com a lista Predefinição, Restaurar personalizações e um botão Definir e um Limpar em cada linha](../assets/shortcuts/keyboard-shortcuts-window.png)

Os atalhos esperam enquanto você digita num campo ou um menu está aberto. Enquanto o ponteiro está sobre uma janela flutuante, como o Gerador de formas, só salvar, desfazer e refazer funcionam, então uma tecla pensada para a janela nunca apaga nem altera seu mapa.

## Ferramentas { #tools }

Aperte a tecla de uma ferramenta de novo para voltar para Selecionar. Esc cancela um arraste ou traço, senão volta para Selecionar, e em Selecionar limpa a seleção.

| Tecla | Ferramenta |
| --- | --- |
| Q | Selecionar, desenhar brushes e mover coisas |
| C | Clip, para cortar brushes ao longo de um plano |
| V | Vértice, para arrastar os cantos dos brushes |
| R | Girar |
| T | Escala |
| Tab | Edita a seleção onde ela está: malhas na ferramenta Malha, brushes na ferramenta Vértice. Nunca converte um brush, *Malha > Editar malha* é quem faz isso |
| Shift+T | Textura, para alinhar texturas nas faces |
| P | Pintar, para pintar cores de vértice nas faces |
| G | Esculpir terreno e displacements |
| Shift+G | Misturar camadas de terreno |
| B | Scatter |
| Shift+E | Volume, para arrastar um volume de trigger |
| Shift+P | Caminho, para colocar uma cadeia de entidades `path_corner` |
| M | Medir distâncias |

## Mouse nas vistas { #mouse-in-the-views }

| Entrada | Resultado |
| --- | --- |
| Arrastar fora da seleção | Desenha um brush |
| Arrastar a seleção | Move. Ctrl copia, Shift trava o eixo, Alt move na vertical em 3D |
| Clique, clique duplo | Seleciona, seleciona o grupo inteiro |
| Clicar de novo logo no mesmo ponto, ou Alt+clique (2D) | Seleciona o próximo objeto sob o cursor, como uma parede sob o teto na vista Topo |
| Ctrl+clique | Adiciona à seleção ou tira dela, nas vistas e no Outliner |
| Shift+clique no Outliner | Seleciona as linhas a partir da última clicada |
| Shift+clique, Ctrl+Shift+clique | Seleciona uma face, adiciona ou tira uma face |
| Shift+arrastar uma face selecionada (3D) | Empurra ou puxa |
| Ctrl+Shift+arrastar uma face selecionada (3D) | Extruda um brush novo |
| Arrastar com o botão direito (3D) | Olhar ao redor. Segure e use WASD para voar, Q e E para descer e subir |
| Arrastar com o botão do meio | Deslocar a vista |
| Alt+arrastar com o botão esquerdo (3D) | Orbitar. Na ferramenta Selecionar, um arraste que começa na seleção ou no gizmo dela a move na vertical |
| Ctrl+roda (Esculpir, Mistura, Pintar, Scatter) | Muda o tamanho do pincel |

## Edição { #editing }

| Tecla | Ação |
| --- | --- |
| Ctrl+Z, Ctrl+Shift+Z ou Ctrl+Y | Desfazer, refazer |
| Ctrl+X, Ctrl+C, Ctrl+V | Recortar, copiar, colar sob o mouse |
| Ctrl+D, Ctrl+Shift+D | Duplicar, duplicar vinculado |
| F2 | Renomeia o objeto selecionado no Outliner |
| Ctrl+R | Repete a última rotação, espelhamento, deslocamento ou duplicação. Na edição de malha, faz um corte em anel |
| Delete ou Backspace | Apagar |
| Ctrl+A, Esc, Ctrl+I | Selecionar tudo, nada, inverter |
| Ctrl+T, Ctrl+Shift+T | Usa os brushes selecionados como estêncil e seleciona o que eles tocam, ou o que fica dentro deles. Os brushes do estêncil são apagados |
| Ctrl+G, Ctrl+Shift+G | Agrupar, desagrupar |
| Ctrl+H, Ctrl+J, Ctrl+Shift+H | Oculta a seleção, isola ocultando todo o resto, mostra tudo |
| Setas, Page Up, Page Down | Desloca um passo da grade |

## Brushes, malhas e texturas { #brushes-meshes-and-textures }

| Tecla | Ação |
| --- | --- |
| Ctrl+K, Ctrl+M, Ctrl+L | Subtração de CSG (escava a seleção do que ela toca), mesclagem convexa, interseção |
| Ctrl+Shift+K | Esvaziar, transforma um brush sólido em uma sala com paredes |
| Ctrl+Shift+B | Gerador de formas, para arcos, cilindros, escadas e outras formas prontas |
| Ctrl+Shift+E, Ctrl+Shift+J | Converter brushes em malha, juntar malhas |
| Ctrl+Shift+W | Mover brushes para o mundo, tirando-os da entidade deles para que virem geometria comum da fase |
| Ctrl+Shift+U | Liga ou desliga o bloqueio de UV, que mantém as texturas presas a um brush enquanto ele se move |
| Alt+H | Ajuste ao hotspot, que encaixa as faces selecionadas no retângulo de uma trim sheet que melhor combina |

## Vista e arquivos { #view-and-files }

| Tecla | Ação |
| --- | --- |
| F ou Ctrl+U | Enquadrar a seleção, ou o mapa inteiro |
| `[` e `]`, ou `-` e `=` | Reduzir à metade ou dobrar a grade |
| F4, F3 | Alterna o sombreamento (com textura, plano, iluminado, wireframe), liga ou desliga a Prévia iluminada |
| Shift+Espaço | Maximiza a vista sob o ponteiro |
| Ctrl+Shift+1 a 9, Ctrl+1 a 9 | Guardar, recuperar marcadores de câmera |
| Ctrl+=, Ctrl+-, Ctrl+0 | Escala da interface maior, menor, padrão |
| F1 | Abre o [manual](interface.md#help-and-the-manual) na ferramenta, aba de painel, entrada de menu ou seção do Inspetor sob o ponteiro, em qualquer outro lugar a paleta de comandos |
| Ctrl+clique numa ferramenta da barra, Cmd+clique no macOS | Abre a página dela no manual |
| Ctrl+Shift+P | Paleta de comandos |
| Ctrl+N, Ctrl+O | Novo mapa, abrir mapa |
| Ctrl+S, Ctrl+Shift+S | Salvar, salvar como |
| Ctrl+Shift+N, Ctrl+Tab, Ctrl+W | Nova aba, próxima aba, fechar aba |
| F5 | Executar o projeto Godot |

> [!NOTE]
> O F2 antes alternava o sombreamento. Agora ele renomeia, como no Godot, no Blender e na maioria dos gerenciadores de arquivos, e o sombreamento passou para o F4. Um atalho que você mesmo definiu na janela de atalhos continua como estava.

## Outras predefinições { #other-presets }

Uma predefinição só muda as teclas abaixo, todo o resto continua como acima. Suas próprias associações ficam por cima de qualquer predefinição, e **Restaurar personalizações** na janela de atalhos as limpa.

| Tecla | Predefinição Hammer |
| --- | --- |
| Shift+S, Shift+X, Shift+V, Shift+A | Ferramentas Selecionar, Clip, Vértice, Textura |
| H, Ctrl+H, U | Ocultar, isolar, mostrar tudo |
| Ctrl+W | Mover brushes para o mundo |
| Ctrl+M | Gerador de formas |
| Ctrl+L | Girar 90° em torno do eixo vertical |
| Ctrl+Shift+M, Ctrl+Shift+L | Mesclagem convexa de CSG, interseção |
| Ctrl+F4 | Fechar aba |

| Tecla | Predefinição Blender |
| --- | --- |
| A, Alt+A | Selecionar tudo, nada |
| X | Apagar |
| Shift+D, Alt+D | Duplicar, duplicar vinculado |
| S | Ferramenta Escala |
| H, Alt+H, / | Ocultar, mostrar tudo, isolar |
| Ctrl+J | Juntar malhas |
| Z | Alternar o sombreamento |
| Shift+A | Gerador de formas |
| F3 | Paleta de comandos |
| F4 | Prévia iluminada |
| Alt+Shift+H | Ajuste ao hotspot |
