---
source: 84383292fb022d6e6369e9ad8b643a67a07d4a67
---

# Vindo do Unity ou do Blender { #coming-from-unity-or-blender }

O TrenchDot é um editor de brushes na linha do TrenchBroom e do Hammer. Um nível começa como blocos sólidos que você desenha numa grade, não como malhas importadas. A maior parte do que você sabe do Unity e do Blender ainda tem lugar aqui, só com outros nomes e teclas. Esta página faz o mapa entre eles, e [Primeiros passos](../getting-started.md) te leva por uma primeira sala.

## Onde ficam as coisas { #where-things-are }

| No Unity ou no Blender | No TrenchDot |
| --- | --- |
| Hierarchy, Outliner | O **Outliner**, com [camadas e grupos](organizing.md) em vez de objetos pai |
| Inspector, painel N | O **Inspetor**. Os campos de posição e tamanho aceitam valores digitados, Enter os aplica |
| Scene view, viewport | A vista 3D mais as vistas Topo, Frente e Lado. Desenhar e alinhar é mais fácil em 2D |
| Formas do ProBuilder, menu Add | Arraste em qualquer vista para desenhar um [brush](brushes.md) de caixa, Ctrl+Shift+B para arcos, escadas e cilindros |
| Modificador Boolean | *Brush > CSG*: subtrair (Ctrl+K), mesclar, intersectar e esvaziar. Eles alteram os brushes na hora, não existe pilha de modificadores |
| Edit mode | A ferramenta **Vértice** para brushes, a ferramenta **Malha** para [malhas](meshes.md), as duas no Tab |
| Prefabs | [Prefabs](organizing.md#prefabs), salvos como o próprio mapa e atualizados em cada instância |
| Duplicata vinculada (Alt+D) | Duplicar vinculado, Ctrl+Shift+D, para que as cópias compartilhem a geometria |
| Unidades, 1 m | 32 unidades de mapa são um metro, o passo de grade padrão é 16 |

Os números exatos vão no Inspetor, à direita. Selecione algo, digite uma nova posição ou tamanho e aperte Enter. A posição é o canto mais baixo da seleção, não o centro nem um pivô.

![O Inspetor com um brush selecionado, sua posição e tamanho como campos X, Y e Z acima da seção Seleção](../assets/coming-from-unity-blender/inspector-position-size.png)

## Teclas que diferem { #keys-that-differ }

As teclas padrão seguem o TrenchBroom, e três delas pegam usuários do Blender de surpresa:

* **G** é a ferramenta de terreno Esculpir, não agarrar.
* **R** e **T** escolhem as ferramentas Girar e Escalar, que agem na seleção com alças em vez de seguir o mouse.
* **Tab** edita a seleção onde ela está.

Para mover algo, arraste em qualquer vista ou use o gizmo na vista 3D, e use o Inspetor para números exatos.

Prefere manter as teclas do Blender? Troque *Arquivo > Preferências > Predefinição de atalhos* para Blender.

![A lista Predefinição de atalhos na janela de Preferências, aberta com TrenchBroom, Hammer e Blender](../assets/coming-from-unity-blender/keymap-preset.png)

Ela acrescenta A para selecionar tudo, X para apagar, Shift+D para duplicar, H e Alt+H para esconder e mostrar, S para a ferramenta Escalar e Shift+A para o gerador de formas. G continua sendo Esculpir, e digitar um número durante um movimento só funciona dentro da ferramenta Malha. A lista completa está em [Outras predefinições](shortcuts.md#other-presets).

Não há vistas do teclado numérico. As vistas Topo, Frente e Lado ficam sempre abertas, Shift+Espaço maximiza a que está sob o ponteiro e *Exibir > Vistas* alterna entre uma, duas e quatro.

## Hábitos que continuam { #habits-that-carry-over }

Na ferramenta Malha as teclas seguem o Blender: 1, 2 e 3 para vértices, arestas e faces, G, R e S com X, Y ou Z para travar um eixo, E para extrudar, Ctrl+R para um corte em laço e Ctrl+B para chanfrar.

Arrastar com o botão direito com WASD voa a câmera 3D como a scene view do Unity, e F enquadra a seleção. Ctrl+Z e Ctrl+Shift+Z desfazem e refazem, e F2 renomeia.
