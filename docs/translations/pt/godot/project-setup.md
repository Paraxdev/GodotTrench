---
source: 0b6155b82bf9e0fff2345e05627d7c675a868912
---

# Configuração do projeto { #project-setup }

Os padrões do addon servem bem para os seus primeiros mapas. Quando você escrever suas próprias entidades ou quiser outra pasta de texturas,
dê ao projeto os próprios recursos, porque uma atualização do addon substitui os padrões.

## Instalando e atualizando o addon { #installing-and-updating-the-addon }

Toda vez que você abre um projeto, o TrenchDot confere o addon dele. Quando `addons/func_godot` está faltando ou tem uma versão diferente
da do editor, a janela *Configurar este projeto* aparece uma vez, e uma nota amarela fica no canto inferior direito da
barra de status. Passe o mouse sobre ela para ver as duas versões, clique nela para trazer a janela de volta.

![A barra de status com a nota amarela "Addon 0.0.9, o editor 0.1.0" sob o mouse e a dica dela](../assets/project-setup/status-addon-mismatch.png)

Você também pode abrir a janela a qualquer momento com *Godot > Instalar ou atualizar addon…*.

![O menu Godot com Instalar ou atualizar addon… sob o mouse](../assets/project-setup/menu-install-update-addon.png)

Ela só oferece o passo de que o seu projeto precisa em seguida:

| O projeto tem | A janela oferece |
| --- | --- |
| Nenhum addon | **Instalar o addon**, depois **Ativar o plugin** |
| Um addon mais antigo | **Atualizar o addon para** a versão do editor |
| Um addon mais novo, ou um sem versão | **Substituir o addon por** a versão do editor |
| O addon, sem estar ativado | **Ativar o plugin**, que o adiciona a `[editor_plugins]` no `project.godot` e não muda mais nada |
| O addon como checkout do git ou submódulo | Nada. Atualize-o com o git, o editor nunca o substitui |

![A janela Configurar este projeto dizendo que o addon 0.0.9 é mais antigo que o editor, com o botão Atualizar o addon para 0.1.0](../assets/project-setup/setup-update-addon.png)

> [!WARNING]
> Feche o Godot antes de atualizar. Ele continua rodando o addon antigo até reiniciar, então o botão fica
> esmaecido enquanto o Live Link vê o Godot com o projeto aberto. Ative o plugin também com o Godot fechado, ou o Godot
> grava a própria cópia do `project.godot` de volta. Marcá-lo no Godot em *Project > Project Settings > Plugins* funciona
> igualmente.

O addon antigo não é apagado, ele vai para `.trenchdot/addon-backups/func_godot-<version>` no seu projeto. O Godot ignora
pastas que começam com ponto, então não importa o backup nem o lista como um segundo plugin. Apague-o quando o novo
addon funcionar. (Nota: o download é descompactado primeiro e só depois trocado, então uma atualização que falhou ou foi cancelada deixa o
addon antigo como estava.)

Se o download falhar, a janela leva às [versões](https://github.com/Paraxdev/TrenchDot/releases) e ao
[itch.io](https://paraxdev.itch.io/trenchdot), que trazem o mesmo `func_godot-trenchdot-addon.zip`. Extraia-o
na pasta do projeto para obter `addons/func_godot`, e substitua a pasta antiga em vez de descompactar por cima.

## Entidades do núcleo e o pacote de entidades de gameplay { #core-entities-and-the-gameplay-entities-pack }

Todo projeto recebe um pequeno núcleo do addon: worldspawn, `func_detail`, `func_illusionary`, `func_geo`, `light`,
`light_spot`, `light_probe`, `env_reflection_probe`, `env_voxel_gi` e `info_player_start`.

Portas, botões, elevadores, trens, triggers, lógica, spawners, teletransportes, path corners, props, efeitos e o resto da
[referência de entidades](../gameplay/entities/README.md) vêm como o pacote de entidades de gameplay. Você o instala quando quiser,
e ele é copiado para `res://trenchdot/entities`:

| Caminho | Guarda |
| --- | --- |
| `gameplay_fgd.tres` | O arquivo FGD do pacote, listando toda definição |
| `definitions/` | Uma definição por entidade: as keys, entradas, saídas e gizmos de editor dela |
| `scripts/` | O GDScript que cada entidade executa |
| `pack.json` | O que foi instalado, para que as atualizações separem as suas edições. Deixe como está |

As cópias pertencem ao seu projeto. Mude um script ou uma definição, apague as entidades que você não usa ou adicione suas próprias
definições ao `gameplay_fgd.tres`. O addon nunca define esses classnames nem classes de script por conta própria, então nada do que você faz
ali conflita com ele, e uma atualização do addon não desfaz isso.

### Instalando e atualizando o pacote { #installing-and-updating-the-pack }

1. Instale o pacote

    Clique em **Instalar entidades de gameplay** na janela *Configurar este projeto*, abaixo do addon.

    ![A janela Configurar este projeto com a seção Entidades de gameplay e o botão Instalar entidades de gameplay](../assets/project-setup/setup-gameplay-entities.png)

    Enquanto o projeto só tem o núcleo, o mesmo botão fica no topo do menu Gameplay e no painel Entidades.

    ![O menu Gameplay com Instalar entidades de gameplay sob o mouse e a dica dele](../assets/project-setup/menu-install-gameplay-entities.png)

2. Confira se funcionou

    A janela deve agora dizer "Instalado em res://trenchdot/entities" em verde, e o editor oferece as novas
    entidades na hora.

    ![A seção Entidades de gameplay depois de instalar, dizendo que adicionou 86 arquivos a res://trenchdot/entities e para reiniciar o Godot se ele estiver com o projeto aberto](../assets/project-setup/setup-gameplay-installed.png)

    Quando um editor do Godot tem o projeto aberto, o TrenchDot pede que ele pegue os arquivos novos, e a janela diz isso. Uma
    cena no [modo ao vivo](live.md#when-live-mode-rebuilds-the-whole-map) então se reconstrói sozinha. Aperte **Build Map**
    nas outras cenas que usam seus mapas e salve-as, já que foram construídas sem o pacote. Quando o Godot não está
    rodando, ou roda um addon mais antigo, a janela diz para reiniciar o Godot se ele estiver com o projeto aberto, depois para construir e
    salvar essas cenas.

3. (Opcionalmente) Atualize depois

    Quando uma atualização do addon traz um pacote mais novo, a janela oferece **Atualizar entidades de gameplay**, e o menu Gameplay o mostra
    com o número de arquivos alterados. Nada é atualizado até você clicar. A atualização adiciona arquivos novos e substitui
    os que você não tocou. Os arquivos que você editou ficam como estão e o editor os nomeia, e os arquivos que você apagou continuam
    apagados.

Mapas antigos, feitos antes de as entidades saírem do addon, continuam funcionando. Quando os mapas de um projeto usam entidades do pacote que nada no
projeto define, o pacote se instala sozinho, tanto quando o editor abre o projeto quanto quando o Godot inicia com o
addon atualizado. Um arquivo `.tdm` mantém toda entidade de qualquer jeito, com ou sem definição.

> [!NOTE]
> O template dentro do addon, `addons/func_godot/gameplay_pack`, tem um arquivo `.gdignore` para que o Godot o ignore.
> Mantenha esse arquivo, ou o Godot acha toda classe de script do pacote duas vezes depois de instalado.

### Qual definição vence { #which-definition-wins }

O FGD do núcleo, `res://addons/func_godot/fgd/trenchdot/trenchdot_fgd.tres`, lê o pacote instalado por conta própria, então
todo FGD construído sobre os padrões do addon recebe o pacote sem listá-lo. Quando duas definições têm o mesmo classname, a
última vence, nesta ordem:

1. As classes do FuncGodot e o núcleo do addon
2. O pacote instalado em `res://trenchdot/entities`
3. As entidades do seu próprio arquivo FGD

Para mudar uma entidade, edite a cópia dela no pacote, ou defina o mesmo classname no seu próprio FGD. Os dois funcionam, mas um
classname definido duas vezes é fácil de esquecer, então o editor avisa: quando o projeto abre, a barra de status nomeia a
definição que vence, e o painel Problemas avisa de cada classe definida em dobro que um mapa usa. Apague uma das duas para
resolver.

## Seus próprios recursos { #your-own-resources }

Três recursos trabalham juntos. O arquivo FGD lista as entidades que seus mapas podem usar. As configurações do mapa dizem a uma construção onde
ficam as texturas e qual FGD ler. A configuração do jogo informa o TrenchDot sobre os dois, para que o editor ofereça as mesmas
entidades e texturas enquanto você edita.

O caminho mais rápido é copiar `demo_fgd.tres`, `demo_map_settings.tres` e `demo_game_config.tres` de
[`godot/demo`](https://github.com/Paraxdev/TrenchDot/tree/main/godot/demo) para o seu projeto e corrigir os caminhos. Para
fazê-los você mesmo, faça isto no editor do Godot:

1. Crie um `FuncGodotFGDFile`. Defina *Base Fgd Files* como `res://addons/func_godot/fgd/trenchdot_default_fgd.tres`, para
   manter o núcleo e o pacote instalado, depois adicione suas entidades em *Entity Definitions*. Sem essa base
   o seu FGD não tem nenhuma delas, e nem o worldspawn constrói com colisão.
2. Crie um `FuncGodotMapSettings`. Defina *Entity Fgd* como o FGD do passo 1 e *Base Texture Dir* como a sua pasta de
   texturas (padrão `res://textures`). Um recurso novo de configurações do mapa começa em `trenchdot_default_fgd.tres`, que tem
   o núcleo e o pacote, mas não as suas entidades.
3. Crie um `TrenchDotGameConfig`. Defina *Fgd File* e *Map Settings* como os dois recursos acima. (Opcionalmente) defina
   *Game Name*, o editor o mostra no canto superior direito da barra de ferramentas, ao lado da pasta do projeto.
4. Em *Project > Project Settings*, aponte `func_godot/default_map_settings` para as suas configurações do mapa e
   `trenchdot/game_config` para a sua configuração do jogo. Todo nó de mapa cujo campo *Map Settings* está vazio a segue. Um
   nó adicionado com um addon mais antigo guardou os padrões do addon nesse campo, então limpe-o lá.

O editor oferece as entidades do FGD da configuração do jogo, mas o Godot constrói com o FGD das configurações do mapa, então mantenha os
dois iguais. Quando diferem, o Godot avisa no painel Output dele cada vez que exporta a configuração do jogo, e o painel
Problemas do TrenchDot nomeia toda entidade que falta no FGD com que o Godot constrói.

> [!TIP]
> Defina *Entity Name Property* nas configurações do mapa como `targetname`, como a demo faz. Os nós construídos então são
> nomeados pelos targetnames deles, o que mantém a árvore de cena legível.

## A configuração do jogo { #the-game-config }

O TrenchDot aprende suas entidades, pastas de texturas e escala de unidades com `res://trenchdot_game.json`, que a configuração do
jogo grava. O addon o exporta de novo sempre que o Godot nota arquivos mudando, e o editor o recarrega em um
segundo. A exportação também nomeia as configurações de mapa padrão e as classes que o FGD delas define, e é assim que o editor avisa
de entidades que o Godot construiria sem definição. Para exportar à mão, use *Project > Tools > TrenchDot: Export
Game Config* no Godot, ou rode sem janela, por exemplo no CI:

```sh
godot --headless --path <project> --script res://addons/func_godot/src/trenchdot/export_game_config_cli.gd
```

## Configurações de projeto { #project-settings }

A maioria dos projetos só muda a primeira e a terceira. As do Live Link importam quando outro programa já usa a
porta.

| Configuração | Padrão | Finalidade |
| --- | --- | --- |
| `func_godot/default_map_settings` | os padrões do addon | Configurações do mapa para todo mapa que não define as próprias |
| `func_godot/default_inverse_scale_factor` | 32 | Unidades do mapa por metro |
| `trenchdot/game_config` | a configuração do addon | A configuração que grava `trenchdot_game.json` |
| `trenchdot/auto_export_game_config` | true | Exporta a configuração do jogo de novo quando arquivos mudam |
| `trenchdot/live_link_enabled` | true | Roda o servidor do Live Link com o qual o TrenchDot fala |
| `trenchdot/live_link_port` | 7842 | Precisa combinar com *Live Link do Godot* nas preferências do editor. As mudanças valem na hora. Entre 1024 e 65535, qualquer outro valor volta para 7842 com um aviso |
| `trenchdot/threaded_build` | true | Converte brushes e chunks de terreno no WorkerThreadPool, para mapas grandes construírem mais rápido |
| `trenchdot/live_chunk_size` | 16.0 | No modo ao vivo, a geometria do worldspawn é dividida em chunks deste tamanho em metros, então uma edição só reconstrói o chunk dela |
| `trenchdot/csharp_entity_dirs` | `res://` | Onde as entidades C# são achadas na hora da construção |
