---
source: 5d131fe08a12db6615a97a4cbd455746685c5f02
---

# Ilha no mar 5: píer e Godot { #sea-island-5-pier-and-godot }

## O píer { #the-pier }

1. Dê a ele uma camada própria

    Clique em **Adicionar camada** no topo do Outliner. A nova camada vira a atual, então o píer cai nela e
    você pode escondê-la ou travá-la separada do terreno.

    ![O Outliner com Adicionar camada sob o mouse e a dica: nova camada, ela vira a camada atual onde os novos objetos entram](../assets/sea-island/outliner-add-layer.png)

2. Construa

    Aperte **Q** para a ferramenta Selecionar, já que um arrasto com a ferramenta Scatter ainda pinta modelos. Depois desenhe brushes
    simples saindo para o mar a partir do meio da praia, com os tamanhos da tabela abaixo.

3. Adicione props

    Vá em *Arquivo > Importar > Prop de modelo (.bbmodel, .glb)…* e escolha modelos da pasta `models/polyhaven` do projeto
    de demonstração: duas `containers/wooden_crate_01` e uma `containers/wine_barrel_01` no deque, uma `nautical/lifebuoy`
    contra um poste e uma `nautical/ocean_buoy` na água. Cada uma cai no cursor 3D, mova-a para o lugar a partir
    dali.

    ![O menu Arquivo com Importar aberto e Prop de modelo (.bbmodel, .glb)… sob o mouse](../assets/sea-island/menu-import-model-prop.png)

| Parte | Tamanho | Material |
| --- | --- | --- |
| Deque | 128 de largura, 1100 de comprimento, 8 de espessura, topo 48 acima da água | `showcase/planks` |
| Postes | 16 unidades de lado, duas fileiras a cada 200 unidades, de 320 abaixo da água até o deque | `showcase/beam` |
| Guarda-corpo | Dois postes curtos na ponta, com 40 de altura | `showcase/beam` |

Um prop referencia o modelo pelo caminho `res://`, então o arquivo precisa estar dentro do projeto Godot aberto. Escolha um de
outro lugar e o editor oferece copiá-lo antes para `res://models`.

![O píer visto do mar](../assets/sea-island/pier.jpg)

## Veja no Godot { #see-it-in-godot }

Salve o mapa dentro do seu projeto Godot e construa-o com um `FuncGodotMap`, veja [Construindo mapas](../godot/building.md).
Cada parte do mapa vira um tipo diferente de nó:

| Parte do mapa | O que você recebe no Godot |
| --- | --- |
| Terreno | Um `TrenchDotTerrain` dividido em chunks, cada um uma malha que mistura as quatro camadas e tem a sua própria colisão |
| Mar | O brush `func_illusionary`, visível sem colisão |
| Conjuntos de árvores, arbustos e pedregulhos | MultiMeshes, mais um corpo estático por modelo com uma forma de colisão para cada instância, todas compartilhando uma forma |
| Conjuntos de grama | MultiMeshes sem colisão que somem com a distância |

![A ilha pronta no Godot](../assets/sea-island/godot-overview.jpg)

![O píer no Godot](../assets/sea-island/godot-pier.jpg)
