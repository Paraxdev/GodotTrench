---
source: d6310713b2a9d65be25e506f0179132b8e0e10fd
---

# Colisão { #collision }

Toda entidade brush do pacote de gameplay e toda entidade de modelo colide sem você configurar nada. Portas, botões,
elevadores, trens e triggers ganham um casco convexo por brush, e `prop_model`, `prop_physics`, `npc_walker` e os props de
scatter ganham uma forma construída a partir do modelo. Você só mexe nas keys abaixo quando o padrão não serve.

## As keys { #the-keys }

Selecione a entidade e ache estas linhas no Inspetor. Passe o mouse sobre uma linha para ler o que cada opção faz.

![O Inspetor de um func_door com o menu de colisão aberto em default, none, box, convex hull, convex decomposition e trimesh, e as linhas collision_layer e collision_mask abaixo](../../assets/collision/inspector.png)

| Key | Padrão | O que faz |
| --- | --- | --- |
| `collision` | default | A forma: default, none, box, convex hull, convex decomposition ou trimesh |
| `collision_layer` | vazio | As camadas de física em que o corpo está, como números de camada, por exemplo `1, 3`. `none` o põe em nenhuma camada |
| `collision_mask` | vazio | As camadas de física que o corpo detecta, na mesma forma. `none` não detecta nada |

Uma camada ou máscara vazia mantém o padrão: a camada e a máscara da definição de entidade para as entidades brush, com os triggers
pegando *Trigger Collision Mask* das configurações do mapa, e as configurações do projeto abaixo para as entidades de modelo.

## Escolhendo uma forma { #picking-a-shape }

| Forma | Entidades brush | Modelos |
| --- | --- | --- |
| default | A forma da classe, um casco convexo por brush | A configuração do projeto, um casco convexo por malha, a menos que mude. `prop_physics` usa uma caixa a partir de `size` |
| none | Sem colisão | Sem colisão |
| box | Uma caixa em volta de todos os brushes | Uma caixa em volta do modelo, a forma mais barata |
| convex hull | Um casco por brush | Um casco por malha, bom para modelos quadrados |
| convex decomposition | Igual a convex hull, um brush já é convexo | Vários cascos que seguem um modelo côncavo como um arco, mais lento de construir |
| trimesh | Cada face, só para corpos estáticos | Cada triângulo, exato mas o mais caro |

O trimesh só funciona em corpos estáticos. Portas, botões, elevadores e trens são corpos animáveis, que contam como estáticos, então
podem usá-lo. Triggers, `prop_physics` e `npc_walker` constroem cascos convexos no lugar, e um trigger configurado como trimesh
mostra um aviso quando o mapa é construído.

Um `prop_model` cujo modelo já tem um corpo de colisão mantém esse corpo e não constrói forma própria.

## Padrões do projeto { #project-defaults }

As entidades de modelo com as keys vazias seguem três configurações em *Project Settings* do Godot, em *Trenchdot >
Collision*. Elas aparecem quando o plugin do TrenchDot está ativado.

| Configuração | Padrão | O que faz |
| --- | --- | --- |
| *Model Shape* | convex | A forma que default representa em `prop_model` e `npc_walker` |
| *Model Layer* | 1 | A camada em que os corpos de modelo ficam |
| *Model Mask* | 1 | As camadas que os corpos de modelo detectam |

Os conjuntos de scatter escolhem a forma na linha *Collision* do próprio conjunto de scatter, veja [Scatter](../../editor/scatter.md).

## Suas próprias entidades brush { #your-own-brush-entities }

Uma entidade brush que você mesmo define colide como o corpo que o *Node Class* dela nomeia. Com *Node Class* vazio ela usa
o corpo que o script dela estende, e sem script, ou com um script que estende `Node3D`, ela vira um `StaticBody3D`. Só um
script que estende outro nó, como um `Marker3D`, a deixa sem colisão, e a construção avisa. Veja
[Entidades próprias em GDScript](../custom-entities.md).

A viewport do editor não desenha formas de colisão. *Debug > Visible Collision Shapes* do Godot as mostra enquanto o
jogo roda, tanto na sessão ao vivo quanto em uma construção completa. As entidades brush e `prop_model` guardam suas formas na
cena construída, `prop_physics` e `npc_walker` constroem as suas toda vez que a cena carrega, do mesmo jeito que carregam o modelo.
Um `prop_model` ou `npc_walker` cuja cena de modelo já tem um corpo de física mantém esse corpo e não constrói
formas próprias.
