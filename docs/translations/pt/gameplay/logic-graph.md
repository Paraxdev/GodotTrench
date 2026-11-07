---
source: 8586c5cf175329bb5dcb3e351f21f472800d70ab
---

# O grafo lógico { #the-logic-graph }

O painel **Lógica** mostra o I/O de entidades do mapa como um grafo. Toda entidade conectada e toda entidade lógica é um nó, e
toda conexão é um fio que sai de um pino de saída à direita de um nó e vai até um pino de entrada à esquerda de outro.

O grafo mostra as mesmas saídas que o Inspetor edita, então uma mudança em um aparece no outro na hora. O mapa, a construção no
Godot e o jogo funcionam exatamente igual, você usando o grafo ou não.

## Abrindo { #opening-it }

Escolha *Exibir > Painéis > Lógica*.

![O menu Exibir com Painéis aberto e Lógica sob o mouse](../assets/logic-graph/menu-panels-logic.png)

Ele abre ao lado de Materiais, abaixo das vistas, e faz esse dock crescer até cerca de metade do meio da janela, já que um grafo
precisa de espaço. Arraste o divisor ou a aba para mudar isso.

![O painel Lógica do mapa de demonstração Night District: botões ligados a portas, um gatilho e um temporizador, com cabeçalhos coloridos por categoria, e uma moldura chamada Hall lights em volta de um gatilho que acende três luzes com atrasos](../assets/logic-graph/graph.png)

## Lendo o grafo { #reading-the-graph }

| O que você vê | O que significa |
| --- | --- |
| Título do nó | O targetname da entidade, senão o nome dela em *Renomear*, senão o classname. Abaixo dele ficam a categoria e o classname |
| Linhas cinzas sob o título | As configurações que diferem dos padrões da entidade, como `max 3`, `wait 2 s` ou `once`, cortadas em duas linhas curtas. Um nó sem elas está nos padrões. O Inspetor mostra todas |
| Cor do cabeçalho | A categoria: o grupo da definição, senão a primeira palavra do classname, então `logic_relay` é *logic*. Veja [Cores](#colors) |
| Pinos | Entradas à esquerda, saídas à direita, vindas da [definição de entidade](entities/README.md) mais qualquer nome que o mapa já use nessa entidade. Um pino preenchido tem um fio. Passe o mouse sobre um pino para ver o tipo e os valores que ele passa |
| Nome de pino amarelo | O nome não está na definição da entidade, o painel Problemas também avisa sobre ele |
| Rótulo do fio | O atraso, `once` ou quantas vezes ele pode disparar, e o parâmetro, quando estão definidos. Passe o mouse sobre um fio para ver a saída, o alvo, a entrada e o atraso por inteiro |
| Nó e fio azuis | Um [alvo](parameters.md#targets) que só o jogo em execução resolve, como `!player`, `@enemies`, um caminho de nó ou um curinga como `door_*` |
| Nó vermelho, fio vermelho tracejado | Um targetname que nenhuma entidade tem, em geral um erro de digitação |

Só entidades conectadas e entidades de lógica e de matemática ganham um nó. Selecione qualquer outra entidade, em uma vista ou no
Outliner, e ela aparece também, pronta para ser conectada.

## Cores { #colors }

As cores ajudam a varrer um grafo grande, mas toda cor também vem com um nome ou uma forma, e as paletas se mantêm distintas para os
tipos comuns de daltonismo.

| Categoria | Cor |
| --- | --- |
| logic | azul |
| math | laranja |
| trigger | verde azulado |
| func, portas, botões e móveis | vermelhão |
| light | amarelo |
| env | azul-petróleo |
| game | azul-celeste |
| prop | rosa |
| npc | roxo avermelhado |
| info | cinza |

| Tipo de pino | Cor |
| --- | --- |
| pulse, um evento sem valor | branco |
| bool | vermelhão |
| int | azul-celeste |
| float | verde azulado |
| string | rosa |
| vector3 | amarelo |
| color | roxo avermelhado |
| node, como o ativador | azul |
| variant, qualquer valor | cinza |

Os pinos que passam um valor são círculos, os pulses são triângulos. Um fio toma a cor da saída de onde parte. Uma categoria sua,
definida com o `group` de uma definição, também ganha uma cor própria.

## Conectando { #wiring }

Arraste de um pino de saída até um pino de entrada. Enquanto você arrasta, os pinos em que o fio se encaixa ganham um anel, e o pino
onde ele terminaria se preenche. Solte e a conexão é adicionada às saídas da entidade de origem.

![Arrastando um fio da saída released de button_1 até a entrada turn_off de hall_light, os pinos em que ele se encaixa mostram anéis](../assets/logic-graph/wire-drag.gif)

Um pulse se encaixa em qualquer pino e um valor se encaixa em um pino do seu próprio tipo, então um nó `activator` não se encaixa em
uma entrada `int`. (Um int também se encaixa em um pino float ou bool e o contrário, e um variant se encaixa em tudo.) Qualquer pino
ainda aceita o fio, os nomes no I/O de entidades só são conferidos quando o jogo roda.

Um alvo sem targetname ganha um no mesmo passo: o nome que você deu a ele com *Renomear* no Outliner, então uma luz que você chamou
de `lamp` vira o alvo `lamp`, senão um livre como `door_2`. Um nome que outra entidade já tem ganha um número depois dele.

Você também pode soltar o fio em outro lugar:

* No título de um nó, para escolher entre todas as entradas dele ou digitar qualquer nome.
* No espaço vazio, para adicionar ali uma entidade lógica, já conectada ao pino de onde você arrastou.

### Adicionando entidades lógicas { #adding-logic-entities }

Clique com o botão direito no espaço vazio, ou aperte **Adicionar** no canto superior esquerdo do painel, para adicionar uma
entidade lógica sem um fio.

![A lista Adicionar: um campo de busca, depois o grupo logic com logic_animate destacado, logic_auto, logic_branch e mais](../assets/logic-graph/add-menu.png)

Sem busca, a lista é agrupada por categoria, logic e math primeiro, depois outras entidades que têm entradas ou saídas. Digite para
buscar em todas: as classes cujo nome começa com o texto ou o contém vêm primeiro, depois as cuja categoria ou descrição o contém. As
setas para cima e para baixo movem o destaque e Enter adiciona essa classe.

As entidades novas são entidades de ponto, e a construção no Godot faz um nó para cada uma, então elas precisam de um lugar no mapa.
Elas vão para uma camada chamada *Logic*, ao lado da entidade a que estão conectadas, ou na frente da câmera da vista 3D. Cada uma
ganha na hora um targetname livre, como `relay_2`, que dá o título ao nó dela. Mude-o no Inspetor, já que *Renomear* só define o
nome no Outliner.

> [!NOTE]
> Sem o pacote de entidades de gameplay não há entidades lógicas para adicionar. O menu Adicionar então se oferece para instalá-lo.

### Editando uma conexão { #editing-a-connection }

Clique em um fio e uma caixa no canto superior direito do grafo deixa você editar a saída, o alvo, a entrada, o parâmetro, o atraso
e as vezes dele. O botão **Excluir** dela, ou a tecla Delete com o ponteiro sobre o grafo, a remove.

![Um fio selecionado de button_1 a hall_light e a caixa Conexão de button_1 no canto superior direito com saída released, alvo hall_light, entrada turn_off, atraso 0,00 s, vezes toda vez e Excluir](../assets/logic-graph/connection-box.png)

Clique com o botão direito em qualquer ponto ao longo de um fio, não só no rótulo, para **Editar conexão** e **Excluir conexão**.
Cada mudança é um passo no *Histórico*, então desfazer funciona como em todo lugar.

### O menu do nó { #the-node-menu }

Clique com o botão direito em um nó para ver o que fazer com a entidade dele.

![O menu de clique direito do nó hall_light: Enquadrar nas vistas, Renomear, Duplicar, Selecionar conectados, Desconectar tudo, Excluir entidade e Simular com a sua saída trocada](../assets/logic-graph/node-menu.png)

| Item | O que faz |
| --- | --- |
| Enquadrar nas vistas | Seleciona-o e leva as vistas até ele, como um clique duplo |
| Renomear | Renomeia a entidade no Outliner. O nó recebe o título do targetname quando há um, então o nome só aparece ali em uma entidade sem |
| Duplicar | Copia a entidade um passo da grade para o lado, com as conexões que saem dela |
| Selecionar conectados | Seleciona o nó e todo nó a que um fio o liga |
| Desconectar tudo | Exclui todo fio que entra ou sai do nó em um passo. As entidades ficam |
| Excluir entidade | Remove a entidade do mapa, veja [Copiar, colar e excluir](#copy-paste-and-delete) |
| Simular | Dispara uma das suas saídas no papel, veja [Simular](#simulate) |

Com vários nós selecionados, Duplicar e Excluir agem em todos eles quando você clica com o botão direito em um dos selecionados.

## Copiar, colar e excluir { #copy-paste-and-delete }

O grafo mostra entidades do mapa, então estas teclas funcionam nas entidades selecionadas, brushes inclusive, enquanto o ponteiro
está sobre o grafo.

| Tecla | O que faz |
| --- | --- |
| Ctrl+C, Ctrl+V | Copiar e colar. A cópia cai um passo da grade do original, com as conexões, já que não há vista sob o ponteiro para colocá-la. Cole com o ponteiro sobre uma vista para posicioná-la ali |
| Ctrl+D | Duplicar, também um passo da grade do original |
| Delete | Remove o fio ou a moldura selecionada. Sem nenhum selecionado, exclui as **entidades selecionadas do mapa**, não só os nós delas. Os fios de outras entidades para uma excluída ficam vermelhos. Desfazer traz tudo de volta |
| Ctrl+A | Seleciona tudo no mapa, não só os nós mostrados |

## Movendo-se { #moving-around }

| Para | Faça |
| --- | --- |
| Zoom | Gire a roda |
| Deslocar | Arraste com o botão do meio ou o direito. Um clique direito sem mover abre um menu, então um deslocamento trêmulo pode fazê-lo aparecer, Escape o fecha |
| Selecionar | Clique em um nó, Shift ou Ctrl com clique adiciona, arraste no espaço vazio para selecionar por caixa. As entidades são selecionadas também nas vistas e no Outliner, e o contrário |
| Enquadrar nas vistas | Dê dois cliques em um nó |
| Mover | Arraste nós. As posições deles são salvas no mapa |
| Arrumar | **Organizar** dispõe os nós selecionados pelo fluxo dos sinais. Com menos de dois selecionados, todo nó volta ao layout automático |
| Ver tudo | **Ajustar**, ou Home com o ponteiro sobre o grafo |
| Ver a seleção | F com o ponteiro sobre o grafo enquadra os nós selecionados |

Os nós que ninguém moveu seguem o layout automático, da esquerda para a direita no fluxo dos sinais, e a vista dá zoom para caber no
painel até você dar zoom ou deslocar por conta própria. Depois que você move um nó ou conecta algo no grafo, os outros mantêm os
seus lugares, e um nó novo aparece ao lado do que ele está conectado. Os nós marcadores azuis e vermelhos não são entidades, então
você não pode movê-los. Eles ficam ao lado do nó que os tem como alvo.

## Molduras { #frames }

Uma moldura é uma caixa com título atrás de um grupo de nós, para manter organizado um grafo grande: as portas de um prédio, um
quebra-cabeça, uma cutscene. Selecione os nós e aperte **C** sobre o grafo, ou **Enquadrar** na barra de ferramentas do painel, e
digite o título dela. A caixa *Hall lights* da imagem no alto é uma.

| Para | Faça |
| --- | --- |
| Mover uma moldura com tudo dentro | Arraste o título dela |
| Redimensioná-la | Arraste o canto inferior direito dela |
| Renomeá-la | Dê dois cliques no título dela, ou selecione-a e aperte F2 |
| Recolori-la ou excluí-la | Clique com o botão direito no título dela. Excluir uma moldura mantém os nós |

As molduras são salvas no mapa como dados do editor, a construção no Godot as ignora.

## Simular { #simulate }

Selecione uma entidade, aperte **Simular** e escolha uma das suas saídas, ou clique com o botão direito em um nó. O grafo numera em
amarelo cada fio que a saída dispararia, e uma lista abaixo do grafo mostra cada passo pelos títulos dos nós, na ordem em que os
passos chegam, com o tempo em segundos depois dos atrasos. **Limpar** encerra a prévia. Clique em um passo para selecionar o fio
dele.

![Simular em button_2: os fios até garage_door e depois até garage_beacon estão amarelos e numerados de 1 a 3, e a lista abaixo do grafo nomeia cada passo](../assets/logic-graph/simulate.png)

Ele segue as regras do tempo de execução: os alvos são achados por targetname, e relays, temporizadores, contadores, portões, portas
e as outras entidades comuns passam a cadeia adiante. Alvos de tempo de execução como `!player` são mostrados, mas não seguidos.

Uma entidade que dispara uma de várias saídas mostra todos os fios que ela poderia disparar. Esses passos, e tudo depois deles,
terminam em `(maybe)` na lista e em `?` no número do fio, e passar o mouse sobre um diz o que ele espera. Um contador alimentado por
`add` ou `subtract` continua até `hit_max` e `hit_min` desse jeito, já que a prévia não conta apertos: o passo está lá, e acontece
quando o contador chega ao seu `max` ou `min`. Um `logic_sequence` dispara os seus passos nos tempos das propriedades `interval`,
`times` e `steps`, então uma cutscene se lê como uma linha do tempo.

Simular funciona no papel, sem o Godot. O que realmente acontece no jogo está em [Depurando as conexões](debugging.md).
