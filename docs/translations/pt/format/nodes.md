---
source: d21533e024f6fb1ea2fed11362601fee5c4efd4d
---

# Nós { #nodes }

O conteúdo de um mapa é uma árvore de nós. Todo nó tem a mesma forma externa, com as chaves nesta ordem:

| Chave | Tipo | Gravada | Significado |
| --- | --- | --- | --- |
| `id` | inteiro | sempre | Único no mapa |
| `type` | string | sempre | `layer`, `group`, `entity`, `brush`, `mesh`, `terrain`, `scatter` ou `instance` |
| campos do tipo | | | As chaves desse tipo de nó, veja abaixo, [Geometria](geometry.md) e [Terreno e scatter](terrain-scatter.md) |
| `label` | string | quando definida | Nome dado com *Renomear*, veja abaixo |
| `graph` | array de 2 números | quando definida | Onde o nó de uma entidade fica no [grafo lógico](../gameplay/logic-graph.md) |
| `hidden` | bool | quando `true` | Escondido no editor |
| `locked` | bool | quando `true` | Não pode ser selecionado nem editado no editor |
| `children` | array de nós | quando não está vazio | Nós filhos, em ordem |

`hidden` e `locked` são só estado do editor, então o Godot ainda constrói um nó escondido. Para deixar algo fora do build, coloque-o numa layer com `omit_from_export`.

`label` é o nome que você deu a um brush, malha, entidade, instância ou terreno no editor, mostrado no lugar de um nome padrão como `brush12`. Layers, grupos e conjuntos de scatter nunca têm uma, eles guardam o nome em `name`.

O Godot só usa `label` em terrenos, ele dá ao nó o nome `terrain_<id>_<label>`. Brushes e malhas se fundem na sua entidade, e uma entidade tira o nome do nó do `targetname`. Editores mais antigos que a chave ainda leem arquivos com ela, mas a descartam ao salvar.

`graph` também é estado do editor. É onde o nó de uma entidade fica no grafo lógico, em pontos do grafo, e só os nós que alguém moveu têm essa chave. O Godot a ignora, e um valor que não são dois números é descartado ao carregar.

## Ids { #ids }

Os ids são inteiros positivos que continuam os mesmos entre salvamentos, para que outros dados possam apontar para um nó. O Live Link envia as edições por id, os conjuntos de scatter nomeiam suas superfícies por id, e o Godot guarda o id de cada nó gerado no metadado `_td_id`. Um id `0` ou duplicado, que só um arquivo editado à mão tem, ganha um id novo ao carregar.

Um prefab é um arquivo `.tdm` separado com ids próprios, então duas cópias dele colidiriam. Para evitar isso, o build do Godot soma um milhão vezes o número da ocorrência ao id de cada nó que uma [instância](#instance) traz. Duas cópias do mesmo prefab nunca compartilham ids, por mais fundo que estejam aninhadas.

## Onde os nós ficam { #where-nodes-go }

| Nó | Pai permitido |
| --- | --- |
| Layer | Só no nível superior |
| Grupo | Layer ou grupo |
| Brush, malha, terreno, conjunto de scatter, entidade ponto, instância | Layer ou grupo |
| Brush ou malha de uma entidade brush | Essa entidade |

Uma entidade com filhos é uma entidade brush, uma porta feita de brushes por exemplo. Brushes e malhas fora de qualquer entidade pertencem ao worldspawn.

## Layer { #layer }

Uma parte de nível superior do mapa, mostrada no Outliner do editor.

| Chave | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `name` | string | obrigatória | Nome mostrado no Outliner |
| `color` | string | obrigatória | `#rrggbbaa`. `#rrggbb` é aceito ao ler |
| `omit_from_export` | bool | `false` | Sempre gravada. Quando verdadeira, o build pula a layer inteira, para geometria de referência que nunca deve chegar ao Godot |

## Grupo { #group }

Nós reunidos para você selecioná-los e movê-los juntos.

| Chave | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `name` | string | obrigatória | Nome mostrado no Outliner |
| `link_id` | inteiro | omitida | Grupos que compartilham um link id são cópias vinculadas, feitas com *Duplicar vinculado* |
| `transform` | 16 números | identidade, omitida | O posicionamento desta cópia em relação às outras, uma matriz 4x4 em ordem column major |

O conteúdo de um grupo é guardado no espaço do mundo, já no lugar. Uma [cópia vinculada](../editor/organizing.md#groups) repete em todas as outras uma edição feita numa cópia, e `transform` é só o que o editor usa para levar a edição de uma cópia para a seguinte. O build do Godot ignora tanto `transform` quanto `link_id`.

## Entidade { #entity }

| Chave | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `classname` | string | obrigatória | Classe de entidade da configuração do jogo, por exemplo `prop_physics` |
| `origin` | `[x, y, z]` | `[0, 0, 0]` | Sempre gravada. Posição de uma entidade ponto |
| `angles` | `[pitch, yaw, roll]` | `[0, 0, 0]` | Sempre gravada. Graus, ordem YXZ como o `rotation_degrees` |
| `properties` | objeto de string para string | omitida | Chaves da entidade, ordenadas por nome |
| `outputs` | array | omitida | Conexões de I/O, veja abaixo |

Os valores das propriedades são sempre strings, seja qual for o tipo no FGD, por exemplo `"travel": "0 168 0"`.

Uma propriedade `angles`, `angle` ou `mangle` vale mais que o `angles` do nó e é lida do jeito Quake: pitch e roll negados, yaw girado em 180 graus. Em classes que começam com `light`, `mangle` é yaw, pitch e roll. Um `angle` de -1 ou -2 aponta direto para cima ou para baixo. O editor desenha a entidade virada para o mesmo lado, e girá-la ali move a propriedade para o `angles` do nó.

O build ignora `origin` e `angles` de uma entidade brush, já que os brushes dela já estão onde devem.

Cada saída é uma [conexão de I/O](../gameplay/io.md): quando esta entidade dispara um sinal, chama um método num alvo.

| Chave | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `output` | string | obrigatória | Sinal desta entidade que aciona a conexão |
| `target` | string | obrigatória | Quem recebe a chamada, veja [Alvos](../gameplay/parameters.md#targets) |
| `input` | string | obrigatória | Método a chamar no alvo |
| `parameter` | string | `""`, omitida | Valor passado ao input |
| `delay` | número | `0`, omitida | Segundos de espera antes da chamada |
| `times` | inteiro | `-1`, omitida | Quantas vezes pode disparar, `-1` para toda vez |

## Instância { #instance }

Uma referência a outro arquivo `.tdm`, posicionada com uma posição e rotação, como o `func_instance` do Hammer. É assim que os prefabs são colocados: o conteúdo vive uma vez no próprio arquivo, e cada instância o mostra.

| Chave | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `path` | string | obrigatória | O mapa referenciado, relativo ao mapa que o referencia, absoluto ou `res://` |
| `origin` | `[x, y, z]` | obrigatória | Onde a instância é colocada |
| `angles` | `[pitch, yaw, roll]` | obrigatória | Graus, ordem YXZ |
| `fixup` | string | `""`, omitida | Prefixo de nome para o conteúdo da instância, veja abaixo |

O build pega os filhos de toda layer do mapa referenciado que não esteja omitida da exportação, e os move por `origin` e `angles`. As projeções de textura se movem junto com a geometria. O worldspawn do mapa referenciado não é usado. As instâncias se aninham até oito níveis de profundidade.

Um build do Godot lê cada mapa referenciado uma vez, e todas as instâncias de um mapa juntas podem somar 2.097.152 valores, sendo um valor um item de array ou entrada de dicionário do mapa referenciado, com cada 8 caracteres de texto contando como mais um. Além disso o build deixa de fora as instâncias restantes e relata um erro. Mapas comuns ficam muito abaixo, só instâncias de instâncias multiplicadas muitas vezes chegam lá.

Com um fixup `p1`, `door` vira `p1-door` em `targetname`, `target`, `destination`, `call_target`, em toda propriedade que o FGD declara como `target_source` ou `target_destination` e em todo `target` de saída. Valores que começam com `!`, `@` ou `/` ficam como estão. Fixups aninhados se somam de fora para dentro, então `b` dentro de `a` dá `a-b-door`, e *Explodir instância* no editor dá os mesmos nomes.
