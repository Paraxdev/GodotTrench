---
source: 1b2a2ceda473607c8db087c552aff86851d08ef2
---

# Entidades que não fazem nada { #entities-that-do-nothing }

Quando o Godot constrói uma entidade sem a definição dela, a entidade é só uma casca. Uma entidade brush como uma porta mantém a malha, mas vira um `Node3D` simples, sem script e sem colisão, e uma entidade ponto vira um `Marker3D` vazio. O mesmo acontece quando uma definição não combina com a entidade, por exemplo uma entidade brush cuja definição não constrói nenhum corpo de física. O TrenchDot e o Godot avisam quando isso acontece, e esta página explica cada mensagem.

## Onde você vê isso { #where-you-see-it }

1. A barra de status

    Salve o mapa no TrenchDot enquanto a cena dele está aberta no Godot. Quando o Godot construiu classes para as quais não tem definição, a barra de status no canto inferior esquerdo as nomeia, junto com as configurações do mapa com que o Godot construiu:

    ![A barra de status dizendo que o Godot reconstruiu 1 nó de mapa, mas res://crypt_settings.tres não tem definição para func_button e func_door, então eles são construídos como nós simples](../assets/entities-troubleshooting/status-missing-classes.png)

2. O painel Problemas

    Abra a aba **Problemas**, ao lado de Outliner e Histórico, à esquerda. Ela lista cada classe que o Godot construiu como um nó simples, com a correção, e os outros avisos dessa construção começam com "Godot:". Clique numa linha para selecionar a entidade.

    ![O painel Problemas com dois avisos, o Godot construiu func_button e func_door como nós simples, cada um dizendo qual arquivo adicionar aos Base Fgd Files](../assets/entities-troubleshooting/issues-missing-classes.png)

    Antes de o Godot construir o mapa, o painel Problemas compara suas entidades com o FGD com o qual o Godot constrói, que o addon exporta junto com a [configuração do jogo](project-setup.md#the-game-config). Uma entidade que esse FGD não tem recebe o aviso "O Godot constrói com ..., cujo FGD ... não tem ...".

    Quando o Godot mostra uma cena que construiu antes de existir uma definição, a barra de status e o painel Problemas dizem "O Godot mostra ..., construído com outras definições de entidades ...". Escolha *Godot > Construir no Godot*, ou aperte **Build Map** no Godot e salve, para construir de novo. **Executar projeto** diz o mesmo sobre as cenas que ele executa.

3. O painel Output do Godot

    Aperte **Build Map** no nó `FuncGodotMap` no Godot. O painel Output na parte de baixo do editor do Godot mostra um aviso `[TrenchDot]` para as classes sem definição e um para cada definição que não combina. Marque *Print Build Report* em *Build Flags* no nó para ver o [relatório de construção](building.md#build-report) inteiro.

## Entidades sem definição { #entities-without-a-definition }

| A mensagem diz | Por quê | Faça isto |
| --- | --- | --- |
| Instale o pacote de entidades de gameplay | Portas, botões, triggers, props e spawners vêm com o pacote, e o projeto não o tem | Escolha *Gameplay > Instalar entidades de gameplay* no TrenchDot, veja [Configuração do projeto](project-setup.md#installing-and-updating-the-pack) |
| Seu FGD não tem o pacote de entidades de gameplay, adicione `trenchdot_default_fgd.tres` aos Base Fgd Files dele | Seu FGD não tem nenhum *Base Fgd Files* | Adicione `res://addons/func_godot/fgd/trenchdot_default_fgd.tres` aos *Base Fgd Files* do FGD que as configurações do mapa nomeiam em *Entity Fgd* |
| O pacote está instalado, mas o *Entity Fgd* é um FGD do addon | As configurações do mapa usam o FGD do próprio FuncGodot ou outro arquivo do addon, que uma atualização do addon substitui | Defina o *Entity Fgd* dessas configurações do mapa como o FGD que sua configuração do jogo usa |
| Uma classe sua não tem definição | Ela está no FGD da sua configuração do jogo, mas não no *Entity Fgd* das configurações do mapa com que o Godot constrói | Defina o *Entity Fgd* dessas configurações do mapa como o FGD que sua configuração do jogo usa, veja [Seus próprios recursos](project-setup.md#your-own-resources) |
| O nó do mapa usa as configurações de mapa padrão do addon, não as das Project Settings | O nó foi adicionado com um addon mais antigo, que guardava as configurações de mapa padrão na cena | Selecione o nó do mapa no Godot e limpe *Map Settings* no Inspetor |
| A cena foi construída com outras definições de entidades | Ela foi construída antes de o pacote ou uma definição existir, e não de lá para cá | Aperte **Build Map** e salve a cena. No [modo ao vivo](live.md#when-live-mode-rebuilds-the-whole-map) o Godot faz isso sozinho |

## Definições que não combinam { #definitions-that-do-not-fit }

Estas vêm do painel Output do Godot e, depois de um salvamento, do painel Problemas como avisos "Godot:". O painel Problemas também sinaliza as duas últimas antes de você salvar.

| A mensagem diz | Por quê | Faça isto |
| --- | --- | --- |
| `func_crusher builds as a Marker3D, so its brushes have no collision` | A definição não tem *Node Class*, e o script dela estende um nó que não é um corpo de física | Defina *Node Class* como `StaticBody3D`, `AnimatableBody3D` ou `Area3D`, ou faça o script estender um deles |
| `func_crusher: Node Class AnimatedBody3D not found, building it as a Node3D` | Um erro de digitação, ou uma `class_name` que o Godot ainda não registrou | Corrija o nome. Para C#, veja [Entidades em C#](../gameplay/csharp.md) |
| `alarm: res://entities/alarm.gd has no @tool` | O editor só executa scripts tool, então as chaves do mapa nunca chegam ao nó | Adicione `@tool` no topo do script |
| `light is a point entity, so its brushes are dropped` | Uma `FuncGodotFGDPointClass` é usada para uma entidade brush | Faça da definição uma `FuncGodotFGDSolidClass` |
| `func_door is a brush entity without brushes` | Uma entidade brush sem nada a partir do que construir | Dê brushes a ela, ou faça da definição uma classe ponto |

## Entidades que são construídas mas não reagem { #entities-that-build-but-do-not-react }

| O que você vê | Por quê | Faça isto |
| --- | --- | --- |
| Os triggers nunca disparam, e o Output do Godot diz que o trigger ignorou seu jogador porque nenhum nó está no `filter_group` dele | Por padrão os triggers só reagem a corpos do grupo `player` na camada de física 1 | Adicione seu jogador ao grupo `player`. Em outra camada, adicione essa camada a *Trigger Collision Mask* nas configurações do mapa |
| Portas e botões nunca se movem | Eles se movem quando o código do seu jogo chama `use()` neles | Veja [Deixe o jogador apertar](../tutorials/button-door.md#let-the-player-press-it) |
| `info_spawner` e `npc_walker` parecem vazios no editor do Godot | Eles só geram ou carregam o modelo no jogo em execução | Veja [Spawners e caminhos](../gameplay/entities/actors.md) |
| O Output do Godot mostra `Parse Error` para um script em `res://trenchdot/entities` | O Godot compilou um script do pacote antes de conhecer as classes que ele usa | Reinicie o Godot, depois aperte **Build Map** e salve |
