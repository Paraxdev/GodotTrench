---
source: ba7a98139fdce301c2c6a3f48c26f62837b5d180
---

# Scatter { #scatter }

A ferramenta **Scatter** (B) pinta árvores, rochas e grama nas superfícies que você escolher, para você cobrir uma encosta em segundos em vez de posicionar cada modelo à mão. O [tutorial da ilha do mar](../tutorials/sea-island-scatter.md) mostra uma configuração completa.

![Pintando uma faixa de floresta pela ilha com a ferramenta Scatter](../assets/scatter/scatter-paint.gif)

## Conjuntos de scatter { #scatter-sets }

Tudo o que você espalha mora num **conjunto de scatter** na própria camada, por exemplo um conjunto para árvores e um para grama. Aperte B e o painel Scatter abre à direita, com um cartão para cada modelo do conjunto ativo.

![O painel Scatter com o conjunto de arbustos ativo: o menu de conjuntos, +, Predefinição, Renomear e um cartão por modelo](../assets/scatter/scatter-panel.png)

| Para | Faça isto |
| --- | --- |
| Começar um conjunto vazio | **+** ao lado do menu de conjuntos |
| Começar de uma predefinição | **Predefinição**, depois escolha um dos dez conjuntos embutidos |
| Adicionar modelos | Arraste-os do painel Modelos, ou **Adicionar modelos…**. **Adicionar props selecionados** pega os modelos das entidades selecionadas |
| Parar de pintar um modelo | Desmarque **pintar** no cartão dele. O que ele já colocou fica |
| Ajustar um modelo | Clique na seta à direita do cartão dele |

![O menu Predefinição aberto com floresta, pinheiros, floresta mista, floresta detalhada, árvores low-poly, arbustos, sub-bosque, rochas, pedregulhos e grama, rochas sob o mouse](../assets/scatter/preset-menu.png)

As predefinições de rochas, pedregulhos, grama, sub-bosque e árvores low-poly usam modelos do Blockbench embutidos no editor. Escolher uma adiciona os modelos de que ela precisa a `res://trenchdot/nature`, offline. As predefinições de floresta, pinheiros, floresta mista, floresta detalhada e arbustos usam as árvores e arbustos glTF do pacote de natureza, um download de cerca de 23 MB. Num projeto sem ele, escolher uma delas abre *Godot > Adicionar conteúdo ao projeto…* nos modelos de natureza em vez de criar um conjunto com modelos faltando. *Terreno > Scatter > Instalar modelos de natureza…*, ou **Instalar modelos de natureza…** no fim do painel, também o abre. Arquivos que já estão no projeto nunca são sobrescritos.

### Cartões de modelo { #model-cards }

Abra um cartão com a seta dele para ajustar aquele modelo.

![Um cartão aberto de bush_round com Escalar, Espaçamento, Alinhar, Inclinação, Afundamento, Giro aleatório e Material](../assets/scatter/model-card.png)

| Configuração do cartão | O que faz |
| --- | --- |
| peso | Com que frequência este modelo é escolhido em comparação aos outros do conjunto |
| Escalar | Um tamanho aleatório entre os dois valores |
| Espaçamento | A menor distância em unidades de mapa até qualquer outra instância |
| Alinhar | 0 deixa o modelo em pé, 1 o inclina para seguir a superfície |
| Inclinação | A maior inclinação aleatória em graus |
| Afundamento | O quanto o modelo é empurrado para dentro do chão, para esconder a base de um tronco ou rocha |
| Giro aleatório | Vira cada instância para uma direção aleatória |
| Material | Desenhado no lugar dos materiais do próprio modelo. Deixado vazio, vale o material do conjunto, se ele tiver um |

## Alvos { #targets }

Um conjunto só pousa nos seus **alvos**, então uma floresta pintada num terreno nunca cobre a casa que está nele. Alt+clique numa vista, ou clique no conta-gotas à direita de *Alvos* e depois clique numa vista, para adicionar ou remover um alvo. Um conjunto sem alvos pega o que o primeiro traço dele atingir. Outro conjunto também pode ser um alvo, é assim que a grama cresce em pedregulhos.

![A seção Alvos do painel Scatter com o terreno como alvo, o conta-gotas no canto superior direito](../assets/scatter/targets.png)

Marcar **Seguir o cursor** ignora os alvos e pinta em qualquer superfície ou prop espalhado sob o brush.

## Pintando { #painting }

As configurações rápidas ficam na barra de opções da ferramenta no canto superior esquerdo: o conjunto, **Pintar** ou **Apagar**, raio, densidade, incl. máx., **Seguir o cursor**, o conta-gotas e **Preencher**. **Painel scatter** traz o painel de volta se você o fechou.

![A barra de opções da ferramenta Scatter com o conjunto de arbustos, Pintar, raio 512 u, densidade 1,00 e incl. máx. 40°](../assets/scatter/scatter-toolbar.png)

| Entrada | Ação |
| --- | --- |
| Arrastar com o botão esquerdo | Pinta |
| Shift+arrastar com o botão esquerdo | Apaga, só do conjunto ativo |
| Ctrl+roda | Redimensiona o brush |
| **Preencher alvos** | Cobre todos os alvos de uma vez, dentro dos limites de declive e altura |

A seção *Brush* do painel guarda o resto.

![A seção Brush do painel Scatter: Raio, Densidade, Declive, Altura, Atenuação, Apenas exposto, Quantidade de apagamento, Semente, Saída e as caixas de seleção embaixo](../assets/scatter/brush-settings.png)

**Densidade** é quantas colocações o brush tenta a cada 64 por 64 unidades, e o espaçamento de cada modelo limita quantas realmente cabem. **Apenas exposto** pula pontos com geometria acima deles, o que mantém a grama longe de baixo de sacadas e telhados. **Manter espaçamento de outros conjuntos**, ligado por padrão, impede que um arbusto cresça atravessando o tronco de outro conjunto.

## No Godot { #in-godot }

O **Tipo** de um conjunto decide o que o Godot constrói.

| Tipo | Construído como |
| --- | --- |
| props | MultiMeshes com colisão e sombras. Modelos cujas cenas carregam scripts são instanciados um a um para os scripts rodarem |
| foliage | MultiMeshes sem colisão nem sombras, escondidos além de 2400 unidades por padrão. Pensado para grama e plantas pequenas |

Selecione o conjunto no Outliner e a linha **Colisão** dele no Inspetor escolhe a forma com que cada instância colide. Conjuntos props novos começam em convexa e conjuntos foliage em nenhum.

| Colisão | Forma por instância |
| --- | --- |
| nenhum | Sem colisão |
| caixa | Uma caixa em volta da instância |
| convexa | Um casco convexo em volta da instância |
| decomposição | Vários cascos convexos que seguem modelos côncavos, como um arco, mais lento para construir |
| trimesh | Cada face do modelo, exato mas o mais caro |

Quando precisar editar props avulsos, **Para entidades** no fim do painel Scatter, ou *Terreno > Scatter > Sets scatter para entidades* para os conjuntos selecionados, troca um conjunto por uma entidade `prop_model` por instância.

![O fim do painel Scatter com Tipo, Pintar, Apagar, Preencher alvos, Limpar instâncias, Para entidades e Instalar modelos de natureza…](../assets/scatter/panel-buttons.png)
