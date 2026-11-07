---
source: 7982e4d62890afb0393dc32addda8d5b4d33f84a
---

# Interface e navegação { #interface-and-navigation }

O TrenchDot segue o TrenchBroom e o Hammer, então se você já usou algum dos dois, a maioria dos hábitos continua valendo. Se não, esta página te mostra o caminho, e [Vindo do Unity ou do Blender](coming-from-unity-blender.md) liga as ideias que você já conhece às daqui.

## A janela { #the-window }

![O editor com uma sala pequena: o Outliner à esquerda, as vistas 3D, Topo, Frente e Lado no meio, o Inspetor à direita e o painel Materiais embaixo](../assets/interface/window.png)

Os menus ficam no alto. Um comando fica sob o tipo de objeto que ele altera, então o CSG está em *Brush* e a escultura de terreno em *Terreno*.

Não achou algo? Aperte Ctrl+Shift+P (ou F1) para abrir a paleta de comandos e digite o nome. Comandos com nome técnico, como *Esvaziar* ou *Bloqueio de UV*, se explicam quando você passa o mouse por cima, e a paleta mostra a mesma linha embaixo da lista. Escape ou um clique fora a fecha sem executar nada.

![A paleta de comandos com esvaziar digitado, Brush: Esvaziar escolhido com seu atalho, e a linha que o explica embaixo](../assets/interface/command-palette.png)

Logo abaixo da barra de ferramentas fica a **barra de opções da ferramenta**. Ela guarda as configurações da ferramenta ativa, ou uma linha sobre como usar uma ferramenta que não tem nenhuma. Passe o mouse sobre o nome da ferramenta ali, ou sobre o botão dela na barra de ferramentas, para ver uma descrição curta.

![A barra de opções da ferramenta com a ferramenta Selecionar sob o mouse, sua dica descrevendo a ferramenta e terminando em F1: manual](../assets/interface/tool-options-tooltip.png)

### O cartão do mapa vazio { #the-empty-map-card }

Num mapa novo e vazio, um cartão na vista 3D diz por onde começar: arraste na vista Topo para desenhar sua primeira caixa, e a vista Topo marca o ponto com um rótulo amarelo. Embaixo ele explica a câmera 3D e leva ao [guia de primeiros passos](../getting-started.md). Quando o projeto aberto tem mapas, **Abrir um mapa deste projeto** os lista, assim você pode abrir um pronto em vez disso.

![O cartão do mapa vazio na vista 3D com Guia de introdução, Abrir um mapa deste projeto e Ocultar dicas, ao lado do rótulo amarelo na vista Topo](../assets/interface/empty-map-card.png)

O cartão some assim que o mapa tem algo dentro. **Ocultar dicas** desliga as dicas, e *Dicas em mapa vazio* em *Arquivo > Preferências* as traz de volta.

### Painéis { #panels }

| Painel | Encaixe | Para que serve |
| --- | --- | --- |
| Outliner | Esquerda | O mapa como uma árvore de [camadas, grupos](organizing.md) e objetos |
| Histórico | Esquerda | Cada edição. Clicar em uma desfaz ela e tudo que veio depois, clicar em uma esmaecida refaz até ela |
| Problemas | Esquerda | Problemas achados enquanto você trabalha, como faces sem material. Clique em um para selecionar o objeto, a maioria oferece uma correção |
| Inspetor | Direita | As propriedades da seleção, como as keys de uma entidade ou a posição e o tamanho exatos de um brush |
| Entidades | Direita | Classes de entidade para arrastar até uma vista |
| Editor de UV | Direita | [Posicionamento exato de textura](uv-editor.md) |
| Scatter | Direita | Os modelos que a ferramenta [Scatter](scatter.md) pinta |
| Materiais | Embaixo | As texturas do seu projeto |
| Modelos | Embaixo | [Arquivos de modelo](models.md) para posicionar |
| Prefabs | Embaixo | Peças prontas da [biblioteca de prefabs](prefab-library.md) para colar |
| Lógica | Começa fechado | A [ligação das entidades como um grafo](../gameplay/logic-graph.md), para conectar entidades e prever o que uma saída dispara |
| Referência | Começa fechado | Código para usar uma classe de entidade nos seus scripts |

Fechou um painel sem querer? Abra-o de novo em *Exibir > Painéis*. *Redefinir layout* no fim do mesmo menu devolve tudo ao lugar de origem.

![O menu Exibir com Painéis aberto, os painéis visíveis marcados e Redefinir layout no fim](../assets/interface/view-panels-menu.png)

## Ajuda e o manual { #help-and-the-manual }

As dicas explicam botões, abas e configurações. Para saber mais, aponte para uma ferramenta, uma aba de painel, uma entrada de menu ou uma seção do Inspetor e aperte **F1**, e a página correspondente deste manual abre. Um Ctrl+clique numa ferramenta da barra de ferramentas (Cmd+clique no macOS) faz o mesmo. Uma dica que termina em *F1: manual* tem uma página por trás, e em qualquer outro lugar o F1 abre a paleta de comandos.

*Ajuda > Manual* abre o início do manual.

![O menu Ajuda com Manual sob o mouse](../assets/interface/help-menu.png)

Quando você já se achar no editor, desmarque *Texto de ajuda* em *Arquivo > Preferências* para esconder as linhas curtas de orientação na barra de opções da ferramenta e nos painéis. As dicas continuam.

(Nota: os links abrem no seu navegador. Se nenhum navegador abrir, a barra de status avisa e o link é copiado para a área de transferência.)

## Vistas { #views }

As vistas são 3D, Topo, Frente e Lado. As vistas 2D olham reto ao longo de um eixo, o que facilita alinhar as coisas. Com algo selecionado, elas escrevem a largura acima do objeto e a altura ao lado dele.

![A vista Frente com um brush de caixa selecionado, 96 escrito acima dele e 48 ao lado](../assets/interface/front-view-size-labels.png)

Um clique numa vista 2D seleciona o objeto mais próximo ao longo da vista, então na vista Topo um teto esconde as paredes embaixo dele. Clique de novo no mesmo ponto em até cerca de um segundo e meio, ou Alt+clique a qualquer hora, para passar ao próximo objeto sob o cursor.

| Faça isto | Para |
| --- | --- |
| Arraste o ponto onde os divisores se cruzam | Redimensionar as quatro vistas de uma vez |
| Shift+Espaço | Maximizar a vista sob o ponteiro, de novo para restaurar |
| *Exibir > Vistas* | Alternar entre uma, duas e quatro vistas, ou trazer de volta uma vista fechada. *Uma vista* mantém a vista em que você trabalhou por último, a vista 3D no começo, e enquanto só uma vista aparece, escolher o nome de outra troca para ela |

![O menu Exibir com Vistas aberto: as quatro vistas marcadas, Uma, Duas e Quatro vistas, e Maximizar vista em Shift+Espaço](../assets/interface/view-views-menu.png)

## Abas { #tabs }

Cada mapa aberto ganha uma aba, e a barra de abas aparece sob a barra de opções da ferramenta quando dois estão abertos. Ctrl+Shift+N (ou o botão **+**) abre uma aba nova, Ctrl+Tab alterna e Ctrl+W fecha uma.

![A barra de abas com room.tdm e untitled.tdm abertos e o botão + sob o mouse, mostrando Nova aba](../assets/interface/map-tabs.png)

*Novo mapa* e a importação abrem numa aba nova quando o mapa atual tem alterações não salvas. Fechar uma aba modificada pergunta se salva, descarta ou cancela, e sair lista todos os mapas não salvos.

## Câmera { #camera }

| Entrada | Vista 3D | Vistas 2D |
| --- | --- | --- |
| Arrastar com o botão direito | Olhar ao redor. WASD move, Q e E descem e sobem, Shift triplica a velocidade | Deslocar |
| Arrastar com o botão do meio | Deslocar | Deslocar |
| Roda | Avançar e recuar, mais rápido quanto mais longe estiver a superfície bem à frente | Zoom na direção do cursor |
| Alt+arrastar com o botão esquerdo | Orbitar, a não ser que o arrasto comece na seleção, que então se move na vertical | |

O WASD só voa enquanto você segura o botão direito, fora isso essas teclas são atalhos. Esqueceu os controles? Passe o mouse sobre a aba de uma vista. Até você olhar ao redor pela primeira vez, a vista 3D também os mostra sob o nome dela enquanto o ponteiro está sobre ela (**Ocultar dicas** no cartão do mapa vazio desliga isso).

**F** enquadra a seleção em todas as vistas, ou o mapa inteiro quando nada está selecionado. Um clique da roda dá zoom numa vista 2D pelo mesmo fator em qualquer zoom, então ir de um terreno grande inteiro até uma única sala leva cerca de 20 cliques, e **F** na sala é mais rápido.

Ctrl+Shift+1 a 9 guarda a câmera 3D como um marcador e Ctrl+1 a 9 volta até ele. Os marcadores ficam salvos no mapa, e desfazer não mexe neles.

## Sombreamento { #shading }

F4 faz a vista 3D percorrer Com textura, Plano, Prévia iluminada e Wireframe, e a vista escreve o modo no canto superior esquerdo. Quando o mapa tem um [bake](light-baking.md), **Iluminação pré-calculada** vem depois da Prévia iluminada. F3 vai direto para **Prévia iluminada** e volta para Com textura, sem passar pelos outros.

![A vista 3D percorrendo o sombreamento com textura, plano, iluminado e wireframe](../assets/interface/shading-cycle.gif)

A prévia iluminada mostra as luzes do mapa, as sombras do sol, o céu e a névoa sem construir no Godot. Ela só desenha as 64 luzes pontuais e spot mais fortes, então um mapa com muito mais pode parecer mais escuro aqui do que no Godot.

## Tema { #theme }

O TrenchDot tem um visual escuro e um claro. Uma instalação nova começa escura.

1. Abra **Arquivo > Preferências**. **Tema** é a primeira linha, no alto da janela.
2. Clique na lista suspensa e escolha **Escuro**, **Claro** ou **Sistema**.

    ![A lista suspensa Tema nas Preferências, aberta com Escuro, Claro e Sistema](../assets/interface/preferences-theme.png)

3. O editor inteiro muda no próximo quadro, sem reiniciar. A escolha é salva com suas outras preferências.

    ![O editor no tema claro, com a barra de menus, o Outliner, a vista 3D e a vista Topo](../assets/interface/theme-light-window.png)

**Sistema** segue o modo claro ou escuro do seu sistema operacional e muda junto com ele. Enquanto ele está escolhido, a lista suspensa mostra *Sistema (Claro)* ou *Sistema (Escuro)* para você ver o que o sistema informou. Onde o sistema não informa um modo, o editor fica escuro.

Você também pode escolher em **Exibir > Tema**, logo abaixo de *Escala da interface*, ou na paleta de comandos como *Exibir: Tema: Escuro*, *Claro* ou *Sistema*. Nenhuma tecla vem atribuída, mas você pode dar uma tecla a cada um no editor de atalhos.

No Claro, as vistas 2D (Topo, Frente e Lado) viram papel, um fundo cinza claro com linhas de grade e arestas de brush escuras, para que as linhas que você desenha continuem legíveis num relance. A vista 3D mantém o entorno escuro nos dois temas, e as telas de textura do editor de UV e do editor de hotspots também. Uma textura tem a mesma aparência em qualquer tema, e um entorno claro mudaria como você julga as cores dela.

<div class="td-mcp" markdown>

Pelo [servidor MCP](../mcp.md), `set_editor` com `"theme": "light"` troca o tema. Os valores são `dark`, `light` e `system`, e um valor desconhecido é um erro que não muda nada. `get_state` informa a escolha como `ui.theme` e o que está na tela como `ui.theme_active`.

</div>

## Idioma { #language }

Os menus, painéis, dicas e mensagens de status vêm em inglês, alemão, francês, espanhol, português, russo, chinês simplificado e japonês. Uma instalação nova começa em inglês.

1. Abra **Arquivo > Preferências**. **Idioma** é a segunda linha, logo abaixo de **Tema**.
2. Clique na lista suspensa e escolha seu idioma. Cada idioma aparece pelo próprio nome, então você acha o seu mesmo quando o editor está num idioma que você não lê.

    ![A lista suspensa Idioma nas Preferências, aberta com English, Deutsch, Français, Español, Português, Русский, 简体中文 e 日本語](../assets/interface/preferences-language.png)

3. O editor inteiro troca na hora, sem reiniciar. A escolha é salva com suas outras preferências.

    ![O menu Arquivo em alemão](../assets/interface/language-file-menu-de.png)

Enquanto outro idioma além do inglês está ligado, a linha mostra por exemplo *Idioma (Language)*, para você sempre achar o caminho de volta. Você também pode escolher um idioma na paleta de comandos como *Exibir: Idioma: Deutsch*, e o mesmo comando funciona em todos os idiomas, já que a paleta também busca os nomes em inglês. Nenhuma tecla vem atribuída, mas você pode dar uma tecla a cada idioma no editor de atalhos.

As traduções foram escritas por IA e nenhum falante nativo as revisou ainda, uma nota sob a linha avisa isso. Um texto que ainda não tem tradução aparece em inglês, e o mesmo vale para um idioma que esta versão não conhece. Se uma tradução soar errada, você pode corrigi-la no [arquivo de localização desse idioma](https://github.com/Paraxdev/TrenchDot/tree/main/crates/td_editor/locales), veja [Traduções](../development.md#translations).

> [!NOTE]
> O manual segue o idioma do editor. Com um idioma escolhido, **F1** sobre uma ferramenta, painel ou menu, e as entradas do menu **Ajuda**, abrem essa página do manual no seu idioma. O botão de idioma no cabeçalho do manual troca o idioma ali. Uma página que ainda não foi traduzida, ou que mudou desde a tradução, aparece em inglês com uma nota no alto. As capturas de tela mostram o editor em inglês, então procure a mesma entrada no seu idioma, ou digite o nome em inglês na paleta de comandos. Classnames de entidade, keys de mapa, formatos de arquivo, o log e o [servidor MCP](../mcp.md) ficam em inglês em todos os idiomas.

<div class="td-mcp" markdown>

Pelo [servidor MCP](../mcp.md), `set_editor` com `"language": "de"` troca o idioma. Os valores são `en`, `de`, `fr`, `es`, `pt`, `ru`, `zh-Hans` e `ja`, em qualquer caixa, e um valor desconhecido é um erro que nomeia todos eles e não muda nada. `get_state` informa a escolha como `ui.language`. Nomes de ferramenta, respostas e erros ficam em inglês em todos os idiomas. Um editor iniciado com `--default-prefs` está em inglês, no que os testes e2e se apoiam.

</div>

## Escala da interface { #interface-scale }

Ctrl+= e Ctrl+- escalam a interface e Ctrl+0 a redefine. Você encontra o mesmo como *Escala da interface* no alto de *Arquivo > Preferências*. Se o monitor informar a escala de tela errada, desmarque *seguir o monitor* logo abaixo.

![O alto da janela de Preferências: Tema em Escuro, Idioma em English, Escala da interface em 100%, Escala da tela com seguir o monitor marcado, e abaixo deles Dicas em mapa vazio e Texto de ajuda](../assets/interface/preferences-scale-and-tips.png)
