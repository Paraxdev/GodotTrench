---
source: d32761c1e423392e984f1e0d2875aadc77d595ce
---

# Malhas { #meshes }

Ao contrário dos brushes, as malhas podem ter reentrâncias, buracos e faces curvas, então use-as para rochas, canos, arcos suaves e outras formas que os brushes não fazem.

## Editar uma malha { #editing-a-mesh }

Selecione os brushes que você quer remodelar e escolha *Malha > Editar malha*. Ela os converte em uma malha e começa a editar na hora.

![O menu Malha com Editar malha sob o mouse, depois Converter brushes em malha (Ctrl+Shift+E), Converter malhas em brushes, Juntar malhas (Ctrl+Shift+J) e os submenus Topologia, Componentes, Normais e sombreamento e Transformar](../assets/meshes/menu-edit-mesh.png)

Com uma malha selecionada, **Tab** faz o mesmo, e Tab de novo te leva de volta aos objetos. Tab em brushes abre a ferramenta Vértice, então uma tecla apertada sem querer nunca transforma um brush em malha. O botão da ferramenta Malha na barra de ferramentas começa a editar sem converter nada. Ctrl+Shift+E só converte, Ctrl+Shift+J junta malhas em uma só.

Enquanto você edita, a barra de opções da ferramenta no alto alterna entre **Vértice**, **Aresta** e **Face**, e tem botões para as operações comuns.

![A barra de opções da ferramenta Malha com Vértice, Aresta e Face, depois Subdividir, Mesclar, Preencher, Excluir, Suavizar e Plano](../assets/meshes/mesh-tool-options.png)

## Teclas { #keys }

As teclas seguem o Blender. Por exemplo, aperte 3, clique na face de cima de uma caixa e aperte E, depois mova o mouse para cima e clique para extrudá-la:

![Uma caixa de malha no modo de face, sua face de cima extrudada para cima em passos de grade](../assets/meshes/extrude-face.gif)

| Tecla | Ação |
| --- | --- |
| 1, 2, 3 | Trabalhar em vértices, arestas ou faces |
| A, Alt+A | Selecionar tudo, selecionar nada |
| Alt+clique, Ctrl+Alt+clique | Seleciona o laço de arestas que passa pela aresta clicada, ou o anel que a cruza |
| Ctrl+L | Seleciona tudo que está ligado à seleção |
| G, R, S | Mover, girar, escalar |
| E | Extrudar nova geometria a partir da seleção |
| I | Inset, uma face menor dentro de cada face selecionada |
| Ctrl+B, Ctrl+Shift+B | Chanfrar arestas ou vértices, cortando a quina |
| Ctrl+R | Corte em laço, um novo anel de arestas atravessando a malha. A roda ou + e - definem o número de cortes |
| K | Faca, clique em dois pontos para cortar ao longo dessa linha |
| M | Mesclar os vértices selecionados no centro deles |
| F | Preencher, uma nova face entre os vértices selecionados |
| X ou Delete | Excluir |
| Shift+D | Duplicar e mover |
| P | Separar a seleção em uma nova malha |
| Alt+F | Inverter as normais, para faces que aparecem pelo lado errado |

Ao mover, girar ou escalar, aperte X, Y ou Z para travar nesse eixo, ou Shift com a letra para travá-lo fora. Digite um número para uma quantidade exata. Clique ou Enter confirma, clique direito ou Escape cancela. A linha de dica no pé da vista lista isso conforme você avança.

## Gizmo { #gizmo }

Dê um clique duplo em um elemento para pôr nele um gizmo de mover, girar e escalar. É o jeito limpo de deslizar um vértice ao longo de um eixo, e funciona nas vistas 2D também. Escape o esconde e mantém a seleção.

O menu *Malha* guarda o resto, como subdividir, triangular, solidificar, espelhar e sombreamento suave.
