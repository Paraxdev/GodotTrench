---
source: 059c4546c8d47d8cdd4e33a900932ff4ff2b3833
---

# Posicionar e conectar { #placing-and-wiring }

## Posicionando entidades { #placing-entities }

Abra a aba **Entidades**, ela fica ao lado do Inspetor, à direita. Toda classe que o seu projeto conhece tem um cartão ali,
agrupado por categoria. Passe o mouse sobre um cartão para ver o que ele faz.

![O painel Entidades com o cartão func_door sob o mouse, o tooltip diz entidade brush, porta deslizante](../assets/placing/entities-panel.png)

Arraste um cartão para qualquer vista para posicioná-lo, ou dê dois cliques nele para soltá-lo no cursor 3D. (Ctrl ou Shift com
clique escolhe vários cartões, arraste-os juntos e eles caem em fila.) O cartão de uma entidade brush vem com uma caixa. Com brushes
selecionados, dê dois cliques nele para envolver esses brushes na entidade em vez disso.

A entidade cai na primeira superfície sólida sob o ponteiro. Volumes de trigger, brushes feitos só de texturas de ferramenta e
outras entidades são ignorados. Enquanto você arrasta, um marcador ciano mostra onde ela vai cair. Uma vista 2D não mostra
profundidade, então ali o marcador traz a altura (ou profundidade) em que ela cai. Na vista Topo o teto de uma sala atrapalha,
então segure Alt ao soltar e a entidade atravessa o teto e cai no chão lá dentro. Nas vistas Frente e Lado, Alt atravessa a
parede mais próxima e põe a entidade no meio da sala.

### Entidades brush { #brush-entities }

Portas, botões e gatilhos são entidades brush, elas tiram a forma dos brushes que você construiu.

1. Construa a forma, por exemplo uma caixa fina onde a porta vai, e selecione-a.
2. Clique com o botão direito em uma vista e escolha uma classe em *Criar entidade de brush*. *Brush > Entidade brush* tem a mesma
    lista.

![O menu de clique direito de uma vista com Criar entidade de brush aberto e trigger_once sob o mouse](../assets/placing/context-create-brush-entity.png)

As mais comuns têm caminhos mais rápidos:

| Ferramenta ou menu | O que faz |
| --- | --- |
| *Gameplay > Portas e móveis* | Uma porta de dobradiça ou deslizante com o seu gatilho, ou um elevador, a partir dos brushes selecionados |
| *Gameplay > Gatilhos* | Um volume de trigger do tamanho da seleção |
| Ferramenta **Volume** (Shift+E) | Arrasta qualquer volume de trigger de uma vez, sem precisar de um brush antes |
| Ferramenta **Caminho** (Shift+P) | Clique a clique, uma cadeia de entidades `path_corner` que trens e caminhantes seguem, Enter termina |

![O menu Gameplay com Portas e móveis aberto: portas de dobradiça, portas deslizantes e Elevador, Plataforma móvel](../assets/placing/gameplay-doors-menu.png)

![A barra de ferramentas com a ferramenta Volume sob o mouse e o seu tooltip, Ferramenta de volume (Shift+E), arraste um volume de trigger, de spawn, de dano, de teletransporte ou de empurrão](../assets/placing/toolbar-volume-tool.png)

### Alças { #handles }

Selecione uma porta, portão, plataforma, botão, gatilho de empurrão, spawner, explosão ou prop de física e ela mostra alças nas
vistas. Arraste-as para definir a dobradiça, o quanto ela abre ou percorre, o raio ou o tamanho, em vez de digitar números. O ângulo
da dobradiça encaixa em passos de 15 graus, segure Shift para graus inteiros.

![Arrastando a alça de ângulo de uma porta de dobradiça na vista Topo, o rótulo vai de 95° até 30°](../assets/placing/door-hinge-handle.gif)

## Conectando { #wiring }

Uma conexão mora na entidade que a dispara. Selecione essa entidade e olhe **Saídas** na parte de baixo do Inspetor. **+ Adicionar
saída** adiciona uma conexão, depois você preenche saída, alvo e entrada. O que cada campo faz está em
[Como funciona o I/O de entidades](io.md).

![A seção Saídas do Inspetor de um botão: saída pressed, alvo front_door, entrada open](../assets/io/inspector-outputs.png)

Para conectar duas entidades mais rápido, selecione as duas e escolha *Gameplay > Lógica > Vincular duas entidades selecionadas…*.
O Inspetor também mostra um botão **Vincular…** enquanto exatamente duas entidades estão selecionadas.

![O menu Gameplay com Lógica aberto e Vincular duas entidades selecionadas… sob o mouse](../assets/placing/menu-link-two-entities.png)

A janela *Vincular entidades* mostra a origem à esquerda e o alvo à direita. Escolha a saída em *Quando*, a entrada em *chamar*, e
aperte **Vincular**.

![A janela Vincular entidades: func_button à esquerda, light à direita, Quando pressed, chamar turn_on, e o tooltip de Trocar](../assets/placing/link-dialog.png)

> [!WARNING]
> A origem nem sempre é a entidade em que você clicou primeiro. Confira os dois nomes no alto e aperte o botão **Trocar** entre
> eles se estiverem invertidos.

O [Grafo lógico](logic-graph.md) mostra toda a ligação de uma vez e conecta entidades arrastando de pino em pino.

## Conferindo as conexões sem o Godot { #checking-wiring-without-godot }

Selecione uma entidade no [Grafo lógico](logic-graph.md#simulate), aperte **Simular** e escolha uma saída. O grafo destaca cada
passo `source.output → target.input` que viria a seguir e os lista com os seus atrasos, então você acompanha uma cadeia longa sem
construir e rodar o jogo. Vermelho marca um alvo que não existe, em geral um erro de digitação em um targetname. Azul marca um alvo
de tempo de execução como `!player` ou `@enemies`, que só existe no jogo e não é seguido adiante.

## Código para qualquer entidade { #code-for-any-entity }

O painel **Referência** mostra GDScript e C# para qualquer classe de entidade. Ele ajuda quando o código do jogo precisa falar com
uma entidade do mapa, por exemplo para abrir uma porta ou ouvir um botão. Abra-o com *Exibir > Painéis > Referência*, com
*Ajuda > Referência de entidades e código*, ou com **Referência de código** no Inspetor, que vai direto à classe da entidade
selecionada. Arraste o divisor entre a lista de classes e os detalhes para redimensionar a lista, dê dois cliques nele para
ajustá-la aos nomes.

![O painel Referência com func_button escolhido: as suas entradas e saídas, e o Uso a partir do GDScript](../assets/placing/reference-panel.png)

Escolha uma classe à esquerda e depois uma aba. *Uso a partir do GDScript* e *Uso a partir do C#* mostram como chamar a entidade,
*Classe GDScript*, *Classe C#* e *FGD (.tres)* são esqueletos para escrever a sua própria. **Copiar** põe o código na área de
transferência. Nas abas de esqueleto, **Criar no projeto** escreve o arquivo em `res://entities/` e deixa em paz um arquivo que já
exista.
