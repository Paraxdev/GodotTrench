---
source: 46b4723cec5ef9109b70a2a54e850458f119c7e9
---

# Executando o Godot pelo editor { #running-godot-from-the-editor }

Salvar, construir e o modo ao vivo falam com o addon pelo Live Link, então funcionam com qualquer editor do Godot que tenha o
projeto aberto. O TrenchDot só precisa achar o executável do Godot para *Godot > Abrir projeto no editor do Godot*, o robô do Godot
na barra de ferramentas e *Godot > Executar projeto* (F5) enquanto nenhum editor do Godot tem o projeto aberto, já que esses começam
o Godot por conta própria.

![O menu Godot com Executar projeto sob o mouse, o atalho F5 à direita e Abrir projeto no editor do Godot logo abaixo](../assets/running-godot/menu-run-project.png)

## Executar projeto { #run-project }

Enquanto um editor do Godot tem o projeto aberto, *Executar projeto* pede a esse editor que salve as cenas abertas e rode a cena
principal, como apertar Play no Godot. O jogo então tem tudo o que o Godot construiu, edições ao vivo incluídas. A barra de status
deve dizer "O Godot salvou suas cenas abertas e executa a cena principal". Se o projeto não tem cena principal, ela diz isso, defina
uma no Godot em *Project > Project Settings > Application > Run*.

Sem um editor do Godot, *Executar projeto* inicia o jogo com o executável do Godot. Ele então roda as cenas como foram
salvas por último no Godot. O mesmo acontece com um addon mais antigo, e a barra de status acrescenta que o jogo pode não ter suas últimas
mudanças no mapa. Salve-as no Godot, ou aperte Play lá.

## Achando o executável { #finding-the-executable }

Vá em *Arquivo > Preferências* e olhe *Executável do Godot*. Quando o TrenchDot achou um, aparece "Usando" e o caminho
embaixo do campo de entrada.

![O campo Executável do Godot nas Preferências com a linha "Usando" e o caminho embaixo dele](../assets/running-godot/prefs-godot-using.png)

Quando aparece "Não encontrado" em amarelo, clique no botão "…" ao lado do campo e escolha o executável do Godot no seu disco,
o que começa com "Godot_v". Faça o mesmo se ele escolheu o Godot errado. Enquanto não acha um, Abrir projeto no editor do
Godot fica esmaecido, e Executar projeto também enquanto nenhum editor do Godot tem o projeto aberto.

![O campo Executável do Godot vazio com o botão … e a linha amarela Não encontrado](../assets/running-godot/prefs-godot-not-found.png)

Sem um caminho que funcione nas Preferências, o editor usa a primeira correspondência de:

1. A variável de ambiente `GODOT`.
2. `godot` ou `godot4` no `PATH`, depois builds baixadas como `Godot_v4.7.2-stable_win64.exe`, as mais novas primeiro. As builds
   Mono ganham quando a pasta do projeto tem um `.csproj`, já que um projeto C# precisa delas. As builds de console são ignoradas.
3. Pastas de instalação comuns: WinGet, Scoop, Steam e `Programs/Godot` no Windows, `~/.local/bin` e Steam no Linux,
   `/Applications/Godot.app` no macOS.

## O botão do Godot { #the-godot-button }

O robô do Godot na ponta direita da barra de ferramentas mostra a conexão. Passe o mouse sobre ele para ver o que faria.

Cinza quer dizer que nenhum projeto está aberto ou que o Godot não foi encontrado, então clicar não faz nada.

![O robô cinza do Godot sob o mouse, a dica dizendo que o Godot não foi encontrado](../assets/running-godot/toolbar-godot-not-found.png)

Normal quer dizer que o Godot foi encontrado. Clique nele para abrir o projeto no editor do Godot.

![O robô normal do Godot sob o mouse, a dica dizendo Abrir projeto no editor do Godot](../assets/running-godot/toolbar-godot-found.png)

Ciano quer dizer que um editor do Godot tem este projeto aberto. Clique nele para trazer esse editor para a frente.

![O robô ciano do Godot sob o mouse, a dica dizendo que o Godot 4.7.2-stable (official) está com este projeto aberto, clique para mostrá-lo](../assets/running-godot/toolbar-godot-open.png)

Um indicador giratório aparece ao lado dele enquanto o Godot está construindo. Se a dica disser que o Godot em execução tem um addon mais antigo, atualize-o
em *Godot > Instalar ou atualizar addon…* para ter o modo ao vivo.

## Confiança no projeto { #project-trust }

O Godot executa os scripts de um projeto assim que o abre, roda ou constrói, e um projeto ou pacote de mapas que você baixou pode
trazer scripts próprios. Então, quando você abre um projeto que o TrenchDot ainda não conhece e que tem código, o TrenchDot pergunta
primeiro.

O TrenchDot primeiro examina o projeto em busca de código. Enquanto faz isso, a barra de status mostra *Procurando código* e
nada é iniciado no Godot sobre o projeto, então um projeto grande leva um momento antes de perguntar ou abrir.

![O aviso perguntando "Você confia nos autores dos arquivos deste projeto?" com a pasta do projeto, o que o modo restrito bloqueia, a caixa da pasta pai e os botões Confiar no projeto e Abrir no modo restrito](../assets/running-godot/trust-prompt.png)

Clique em **Confiar no projeto** para os seus próprios projetos e os de pessoas em quem você confia. Marque *Confiar nos autores de todos os arquivos da
pasta pai* antes para confiar em todo projeto dessa pasta, como aquela onde você guarda todos os seus jogos. Sem isso
você confia só naquele projeto, e um projeto descompactado dentro dele, como um pacote de mapas com o próprio `project.godot`,
não recebe a confiança junto.

Clique em **Abrir no modo restrito** quando não tiver certeza. Você ainda pode ver e editar os mapas, mas o TrenchDot não
inicia o Godot sobre o projeto, não constrói no Godot nem envia edições ao vivo. *Modo restrito* então aparece em amarelo à direita da
barra de status, clique nele para responder à pergunta de novo.

![O chip amarelo Modo restrito na barra de status, a dica dizendo que o TrenchDot não executa este projeto no Godot até você confiar nele](../assets/running-godot/restricted-mode-chip.png)

No modo restrito, *Executar projeto*, o robô do Godot, *Construir no Godot* e ligar o modo ao vivo perguntam "Isso executa seus
scripts. Confiar e continuar?", e seguem em frente quando você clica em **Confiar no projeto**.

Abrir um mapa que pertence a outro projeto em que você não confiou mantém o projeto aberto, e o aviso acrescenta **Manter o projeto
atual** aos botões. Um `project.godot` dentro da pasta do projeto aberto é ignorado, do jeito que o Godot o ignora.
Quando um mapa de fora do projeto aberto roda GDScript próprio, a linha de status diz isso e o painel Problemas
mostra onde.

Projetos em *Projetos recentes* de antes de o TrenchDot perguntar sobre confiança contam como confiáveis, e o mesmo vale para um projeto com o qual você
inicia o editor por `--project`, durante essa sessão.

> [!NOTE]
> O modo restrito cobre só o que o TrenchDot inicia. Abrir o projeto no Godot você mesmo executa os scripts como de costume.

### Projetos sem código { #projects-without-code }

Um projeto que não tem código abre sem a pergunta, já que o Godot não tem nada nele para executar. Isso vale para uma pasta vazia,
um projeto recém-criado, ou um só com o addon do TrenchDot e o pacote de entidades de gameplay que ele instalou. Código aqui
quer dizer scripts, projetos C#, bibliotecas nativas, cenas e recursos com scripts dentro, autoloads, plugins de editor e
mapas cujo `logic_script` guarda GDScript. O addon e o pacote só contam como do próprio TrenchDot enquanto todo arquivo
neles for o que esta versão instala, então uma cópia editada ou mais antiga conta como código, e um projeto grande demais para ser examinado
rápido também.

O TrenchDot examina de novo toda vez que abre o projeto e não lembra de nada, então pergunta quando o projeto ganhou
scripts. Para manter o seu projeto confiável, salve um mapa nele com *Arquivo > Salvar* enquanto ele não tem código. A partir daí
o TrenchDot confia nele, também depois de você adicionar scripts no Godot. Salvar um mapa em um projeto no modo restrito não confia
em nada.

### Projetos confiáveis { #trusted-projects }

Abra *Arquivo > Preferências…* e olhe *Projetos confiáveis* no fim para ver todo projeto e pasta em que você confia.
Clique em **Remover** ao lado de um para retirar a confiança. Quando é o projeto aberto, ele entra no modo restrito na hora
e o chip amarelo aparece na barra de status, a menos que o projeto não tenha código.

![A parte Projetos confiáveis das Preferências listando dois projetos e uma pasta com todos os projetos dentro dela, cada um com um botão Remover, a dica de um deles dizendo que o TrenchDot pergunta de novo antes de o Godot executar os scripts aqui](../assets/running-godot/trusted-projects.png)

<div class="td-mcp" markdown>

## MCP { #mcp }

Um agente nunca recebe o aviso no caminho. `open_project` e `get_state` informam `restricted`, e as ferramentas que fariam
o Godot executar ou construir o projeto falham com um erro até você confiar nele pelo chip. `map_file` `open` mantém o
projeto aberto para um mapa de um projeto sem confiança e devolve esse projeto como `other_project`.

`trust` nos dois resultados diz por que um projeto não está no modo restrito: `stored`, `session` para `--project`, ou `no_code`.
Salvamentos que um agente faz, com `map_file`, `run_action` ou dentro de `run_script`, nunca guardam confiança, então um projeto em que um agente
trabalha sem código é perguntado de novo quando ganha scripts.

</div>
