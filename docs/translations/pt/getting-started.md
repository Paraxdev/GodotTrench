---
source: 18fb3370f830b9fda2728d12b95c3b5cef06be64
---

# Primeiros passos { #getting-started }

Passos 1 a 3: a primeira configuração. Se você já tem um projeto Godot configurado com o TrenchDot, pode pulá-los.

Passos 4 e 5: criar a primeira sala no TrenchDot e vê-la no Godot.

## O que você precisa { #what-you-need }

* Godot 4.7 ou mais novo, a versão .NET se você quiser escrever entidades em C#
* A [versão do TrenchDot](https://github.com/Paraxdev/TrenchDot/releases/tag/beta) mais recente

## Confirme que o TrenchDot enxerga o executável do Godot { #ensure-trenchdot-can-see-the-godot-executable }

Confirme que o TrenchDot enxerga a sua instalação do Godot. Para conferir se tudo foi detectado direito, vá em
*Arquivo > Preferências… > Executável do Godot*.

Se o TrenchDot achou o Godot sozinho, a linha embaixo do campo de texto já diz "Usando" e o caminho. Se em vez disso
ela diz "Não encontrado", você precisa localizar o executável no seu disco.

Para isso, clique no botão "…" ao lado do campo de texto

![O botão … ao lado do campo Executável do Godot](assets/getting-started/godot-path-browse.png)

e procure o executável que começa com "Godot_v".

Se você escolheu o executável certo, a linha deve dizer "Usando" e o caminho que você acabou de escolher.

![A linha embaixo do campo de texto, dizendo Usando e o caminho do executável do Godot](assets/getting-started/godot-path-using.png)

## 1. Crie um projeto no Godot { #1-create-a-project-in-godot }

Abra o Godot e clique em "Create" no canto superior esquerdo.

![O gerenciador de projetos do Godot, o botão Create com um ícone de mais no canto superior esquerdo](assets/getting-started/godot-create-project.png)

Escolha um nome para o projeto, neste tutorial será TrenchDotTest, e (opcionalmente) selecione uma pasta onde o projeto
vai ficar.

Depois clique em "Create".

## 2. Abra o projeto no TrenchDot { #2-opening-the-project-within-trenchdot }

Escolha *Godot > Abrir projeto do Godot…*

![O menu Godot no TrenchDot com Abrir projeto do Godot… destacado](assets/getting-started/menu-open-godot-project.png)

ou o botão **Abrir projeto do Godot…** no canto superior direito da barra de ferramentas

![O botão Abrir projeto do Godot… no canto superior direito da barra de ferramentas do TrenchDot, que mostra o nome do projeto quando há um aberto](assets/getting-started/toolbar-open-godot-project.png)

e escolha a pasta do projeto ou qualquer pasta dentro dela. O editor o reabre na próxima vez que iniciar.

## 3. A tela de boas-vindas { #3-the-welcome-screen }

Assim que o TrenchDot detecta que é um projeto Godot novo, ele mostra a janela *Configurar este projeto*.

![A janela Configurar este projeto com as seções do addon, das entidades de gameplay e do conteúdo pronto](assets/getting-started/setup-window.png)

Aqui você pode baixar o addon, que o Godot precisa para transformar seus mapas em cenas e para conversar com o TrenchDot, e
conteúdo extra, se quiser. Claro que você também pode começar sem nada.

Algumas entidades básicas sempre vêm incluídas, como um simples `info_player_start` e `light`.

1. Instale o addon

    Clique no botão "Instalar o addon" no alto para baixar o fork do FuncGodot feito pelo TrenchDot para o seu projeto,
    isso é essencial para o TrenchDot.
    (Nota: isso baixa dos releases do GitHub. Se você não tem acesso à internet, baixe primeiro
    `func_godot-trenchdot-addon.zip` e extraia na raiz do projeto para instalar à mão.)

    Depois disso, clique em "Ativar o plugin" para o Godot carregá-lo.
    (Nota: feche o Godot antes, se ele estiver com o projeto aberto, senão o Godot grava de volta a própria cópia do `project.godot`
    e o plugin continua desligado.)

2. Instale as entidades de gameplay

    Clique em **Instalar entidades de gameplay** embaixo do addon.

    ![A seção Entidades de gameplay da janela Configurar este projeto com o botão Instalar entidades de gameplay](assets/getting-started/setup-gameplay-entities.png)

    Portas, botões, elevadores, triggers, props e spawners vêm com este pacote. Sem ele, o Godot os constrói como formas
    simples, sem colisão, que não fazem nada. Os auxiliares de porta e trigger no menu Gameplay instalam o pacote
    sozinhos na primeira vez que você os usa, mas instalá-lo agora, antes de o Godot abrir o projeto pela primeira vez, significa que
    nenhuma cena é construída sem ele.

3. (Opcional) Conteúdo extra

    O TrenchDot tem conteúdo opcional que você pode carregar de antemão: texturas e modelos CC0 do Poly Haven e do ambientCG, além de modelos
    feitos para o TrenchDot. As licenças e uma lista de onde cada coisa vem estão incluídas.

    Se você não quer conteúdo extra, escolha "Nada extra", as entidades mínimas continuam incluídas.

4. Abra o projeto uma vez no Godot

    Com o plugin ativado, abra o projeto no Godot (a janela tem um botão "Abrir projeto no editor do Godot" para isso). O addon então cria um arquivo chamado `trenchdot_game.json` na
    raiz do projeto. É dele que o TrenchDot aprende quais são suas texturas e entidades, e o TrenchDot o carrega em um segundo.

    O addon o grava de novo sempre que arquivos mudam, então não é preciso editá-lo à mão. Até ele existir, a barra
    de status diz que não há `trenchdot_game.json`, e o editor oferece as entidades do núcleo, mais as entidades de gameplay,
    se você as instalou.

Para adicionar entidades suas, veja [Entidades próprias em GDScript](gameplay/custom-entities.md) e
[Entidades em C#](gameplay/csharp.md). [Configuração do projeto](godot/project-setup.md) explica os recursos que as listam.

## 4. Construa uma sala { #4-build-a-room }

O editor tem 4 vistas principais: uma vista 3D em que você pode voar como no Godot, e 3 vistas wireframe de cima,
de frente e de lado.

Brushes e operações dependem do tamanho da grade. Na barra de ferramentas você define o tamanho da grade (menor é mais fino).

![A configuração do tamanho da grade na barra de ferramentas, definida como 16](assets/getting-started/toolbar-grid-size.png)

Em um nível vazio aparece um pequeno texto de ajuda explicando como desenhar. Você pode escondê-lo com "Ocultar dicas"
ou simplesmente começar a desenhar, ele some assim que o mapa tem algo dentro.

![Um mapa novo: o cartão na vista 3D diz para arrastar na vista Topo para desenhar a primeira caixa, e a vista Topo mostra um rótulo amarelo "Arraste aqui para desenhar sua primeira caixa"](assets/getting-started/empty-map.png)

1. Desenhe sua primeira caixa

    Com a ferramenta Selecionar (Q), arraste em um espaço vazio de qualquer vista.
    Você deve ver uma caixa sendo desenhada. Mantendo o botão esquerdo pressionado, mova o mouse para formar um retângulo.

    ![As quatro vistas com a primeira caixa desenhada, a vista Topo mostrando o tamanho dela, 512 por 256](assets/getting-started/draw-first-box.png)

2. Adicione altura

    Na vista Lado, segure o mouse sobre a aresta de cima da caixa até ela ficar amarela, mantenha o botão esquerdo pressionado e
    mova o mouse para cima até a caixa ter 256 unidades de altura.

    ![A vista Lado com a aresta de cima da caixa arrastada para cima, 256 escrito acima e ao lado dela](assets/getting-started/side-view-depth.png)

3. Esvazie a caixa

    Pressione Ctrl+Shift+K ou use o botão Esvaziar na barra de ferramentas.

    ![A barra de ferramentas com o botão Esvaziar sob o mouse e a dica dele](assets/getting-started/toolbar-hollow.png)

    A caixa agora está dividida em paredes, um piso e um teto, o que é ideal para uma sala pequena ou um
    corredor. Agora, usando os controles de voo (segure o botão direito do mouse e use WASD), voe para dentro da caixa vazia.

4. Corte uma porta

    Com Escape você desmarca os brushes selecionados. Quando nada estiver selecionado, desenhe na vista Frente um retângulo vertical menor dentro
    da caixa, com a forma de uma porta.

    ![O brush da porta dentro da caixa vazia, 64 de largura e 96 de altura na vista Frente](assets/getting-started/doorway-brush.png)

    Depois, na vista Lado, estenda um lado do brush da porta para ele ter profundidade, atravessando toda a parede.

    ![Arrastando um lado do brush da porta na vista Lado para ele atravessar a parede](assets/getting-started/doorway-extend.gif)

    Agora, com o brush selecionado, pressione Ctrl+K ou use o botão Subtrair na barra de ferramentas.

    ![A barra de ferramentas com o botão CSG subtrair sob o mouse e a dica dele](assets/getting-started/toolbar-subtract.png)

    Você deve ver o brush sumir, e a porta fica cortada na parede.

    ![A parede na vista 3D com a porta cortada](assets/getting-started/doorway-cut.png)

5. Aplique uma textura

    Você deve ter 5 texturas incluídas para prototipagem. Para aplicar uma textura em só uma face (por exemplo, o batente da porta), arraste
    a textura da aba Materiais até a face (segure Shift para cobrir o brush inteiro).

    ![Arrastando dev/blue da aba Materiais até uma face, com a prévia do que será solto](assets/getting-started/texture-drop.png)

6. Salve o mapa

    Para ver as mudanças no Godot, precisamos salvar o mapa. Pressione Ctrl+S ou use *Arquivo > Salvar*.

    A janela de salvar abre na pasta do seu projeto Godot. Salve em qualquer lugar dentro do projeto, uma pasta `maps` na
    raiz do projeto mantém tudo organizado, por exemplo `res://maps/room.tdm`.
    (Nota: o nome da pasta é com você, mas mantenha o mapa dentro do projeto, para o Godot poder carregá-lo pelo caminho
    `res://`.)

    Depois de salvo, ele aparece em *Arquivo > Mapas no projeto*.

    ![O menu Arquivo com Salvar e Mapas no projeto, que lista room em res://maps](assets/getting-started/file-menu-maps-in-project.png)

## 5. Veja no Godot { #5-see-it-in-godot }

No Godot, um nível é uma cena: uma árvore de nós, salva como arquivo `.tscn`, em que cada nó é uma coisa, como uma malha, uma
luz ou uma câmera. O Godot escreve caminhos dentro do projeto com `res://`, então `res://maps/room.tdm` é o arquivo
`maps/room.tdm` na pasta do seu projeto. O mapa é transformado em nós por um nó do addon, o `FuncGodotMap`.

1. No editor do Godot, escolha *Scene > New Scene* e clique em **3D Scene** no painel Scene à esquerda.
2. Pressione Ctrl+A (*Add Child Node*), procure `FuncGodotMap` e adicione.
3. Com o novo nó selecionado, defina *Local Map File* como o seu `.tdm` no Inspetor do próprio Godot, o painel à direita do
   editor do Godot. Ele funciona como o Inspetor do TrenchDot, mas pertence ao Godot.
4. Pressione **Build Map** no alto desse Inspetor e depois salve a cena com Ctrl+S.

Brushes viram malhas com colisão e entidades viram nós com seus scripts, salvos junto com a cena.
[Construindo mapas](godot/building.md) explica as outras configurações do nó. Se uma entidade sair como uma malha pura, sem
colisão, a barra de status do TrenchDot diz o motivo, veja [Entidades que não fazem nada](godot/entities-troubleshooting.md).

Deixe essa cena aberta no Godot e cada salvamento no TrenchDot a reconstrói, na hora se a aba dela for a atual,
ou quando você voltar a ela. Salve a cena no Godot para guardar o resultado. Se um salvamento não reconstruir, a barra
de status diz o motivo, veja [Reconstruir ao salvar](godot/live.md#rebuild-on-save).

> [!WARNING]
> Uma construção apaga primeiro os filhos do `FuncGodotMap`. Coloque seus próprios nós ao lado do nó do mapa, ou
> em um [overlay](godot/overlays.md).

## Experimente o projeto de demonstração { #try-the-demo-project }

A pasta `godot` do repositório, que também vem no release como `trenchdot-demo-project.zip`, é um projeto com o addon
instalado. A cena principal dele executa os mapas de vitrine, salvos em `godot/demo/maps/showcase`. Abra o projeto no
editor e escolha um em *Arquivo > Mapas no projeto* para ver como um mapa pronto é montado.

A opção de demo em *Godot > Adicionar conteúdo ao projeto…* coloca os mesmos mapas e cenas no seu próprio projeto. No Godot,
abra `res://demo/demo.tscn` e pressione F6 para executá-los. Abertos no TrenchDot a partir do seu projeto, eles aparecem sem as
texturas, porque usam a pasta de texturas da demo, `res://demo/textures`, e não a do seu projeto. O projeto de demonstração
os mostra como foram construídos.

| Tecla | Ação |
| --- | --- |
| WASD, Space, Shift | Andar, pular, correr |
| E | Usar portas e botões |
| 1 a 5 | Alternar entre os mapas de vitrine |
| F3 | Mostrar o overlay de depuração de I/O, um registro de cada input e output quando dispara, com os volumes de trigger desenhados |

## Próximos passos { #next }

Para fazer a sala fazer algo, leia [Como funciona o I/O de entidades](gameplay/io.md) e construa
[Botão abre uma porta](tutorials/button-door.md).

O seu próprio jogador precisa de duas coisas para as entidades do pacote. Triggers só reagem a corpos no grupo do Godot `player`
na camada de física 1, então coloque seu jogador lá. Portas e botões se movem quando o código do seu jogador chama `use()` naquele para o qual
ele olha, veja [Deixe o jogador apertar](tutorials/button-door.md#let-the-player-press-it).

<div class="td-mcp" markdown>

## MCP { #mcp }

A pergunta sobre conteúdo não abre sozinha em um editor controlado por um cliente MCP: a primeira chamada de ferramenta que não seja
`get_state`, `screenshot` e `simulate_input` a fecha, e um projeto aberto com `open_project` não recebe a pergunta.
`project_content` adiciona o mesmo conteúdo, veja [MCP](mcp.md).

</div>
