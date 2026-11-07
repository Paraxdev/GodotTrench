---
source: 15ac2f37b7d278fa849fb066b91734f671f35658
---

# Salvar, modo ao vivo e hot reload { #saving-live-mode-and-hot-reload }

## Reconstruir ao salvar { #rebuild-on-save }

Deixe a cena com o seu `FuncGodotMap` aberta no editor do Godot e é só salvar no TrenchDot com Ctrl+S. O Godot
reconstrói essa cena na hora, desde que a aba dela seja a que o Godot mostra e o nó do mapa tenha *Auto Rebuild On Save*
ligado. Uma cena em outra aba que usa o mapa é reconstruída quando você muda para a aba dela. Fechar e reabrir uma aba ou
recarregar uma cena não quebra isso. Salvar um [prefab](../editor/prefab-library.md) reconstrói os mapas dessa cena
que o usam, direto ou por meio de outro prefab.

A barra de status no canto inferior esquerdo diz o que o Godot fez. Quando deu certo, ela deve dizer:

![A barra de status dizendo "Link ao vivo: O Godot reconstruiu 1 nó(s) de mapa"](../assets/live/status-rebuilt.png)

Salve a cena no Godot para guardar o resultado. Se a barra de status disser outra coisa, a mensagem explica o motivo:

![A barra de status dizendo que room.tscn usa room.tdm mas não é a aba de cena atual no Godot, ela é reconstruída ao mudar para ela](../assets/live/status-not-current-tab.png)

| A barra de status diz | Faça isto |
| --- | --- |
| A cena não é a aba de cena atual no Godot | Mude para essa aba, ela é reconstruída então |
| A aba de cena atual não tem um `FuncGodotMap` para o mapa | Abra a cena que constrói o mapa, ou ligue *Auto Rebuild On Save* no nó de mapa dela |
| O Godot não tem cena aberta | Abra no Godot a cena com o mapa, e se ela já está aberta, continue lendo |
| O Godot não está rodando com o addon do TrenchDot | Abra o projeto no editor do Godot com o addon ativado |
| O Godot reconstruiu o mapa, mas as configurações do mapa não têm definição para algumas classes, então elas são construídas como nós simples | Adicione as definições, veja [Entidades que não fazem nada](entities-troubleshooting.md) |
| O Godot reconstruiu o mapa com avisos, veja o painel Problemas | Leia-os no painel Problemas, onde começam com "Godot:" |

Só um editor do Godot pode ficar com a porta do Live Link. Um segundo editor com o mesmo projeto avisa no painel Output dele que
o Live Link não conseguiu escutar, e o TrenchDot continua falando com o primeiro, que pode estar sem cena aberta. Feche o
editor que você não usa e o outro assume o link em poucos segundos. (Nota: *Godot > Abrir projeto no editor do
Godot* traz para a frente um editor que já tem o projeto, em vez de iniciar um segundo.)

## Modo ao vivo { #live-mode }

O modo ao vivo empurra suas edições para a cena aberta no editor do Godot antes de você salvar, para você julgar iluminação e escala
no renderizador de verdade.

1. Abra a cena no Godot

    O mapa precisa ter sido salvo uma vez dentro do projeto, e a cena aberta no Godot precisa de um `FuncGodotMap` que o use.

2. Ligue

    Clique no botão de link na ponta direita da barra de ferramentas, ao lado do robô do Godot, ou use *Godot > Modo ao vivo*. O
    botão acende, e a dica dele diz o que ainda está esperando, se houver.

    ![A barra de ferramentas com o botão de link do modo ao vivo ligado e a dica dizendo que as edições chegam ao Godot antes de você salvar](../assets/live/toolbar-live-mode-on.png)

3. Edite como de costume

    Toda mudança aparece no Godot na hora.

    ![Edições no TrenchDot aparecendo no editor do Godot antes de salvar](../screenshots/TrenchDot-demo-live-view.gif)

O modo ao vivo precisa do Live Link do Godot, que vem ligado por padrão. Os dois ficam em *Arquivo > Preferências*, onde *Modo ao vivo do Godot*
é o mesmo interruptor do botão da barra de ferramentas.

![As linhas Live Link do Godot, Modo ao vivo do Godot e Recarga a quente do jogo em execução nas Preferências](../assets/live/prefs-live-link.png)

Arrastar move os nós enquanto você arrasta, e a maioria das edições reconstrói só o que tocou, então a prévia acompanha. Usar *Alternar omitir
na exportação* em uma camada e editar instâncias de prefab reconstroem o mapa inteiro, assim que você faz uma pausa.

A prévia é quase igual a uma construção completa, mas nem sempre idêntica. Salvar sempre faz uma construção completa, e sair sem
salvar manda o Godot de volta ao arquivo salvo, então nada que você tentou no modo ao vivo fica a menos que você salve.

### Quando o modo ao vivo reconstrói o mapa inteiro { #when-live-mode-rebuilds-the-whole-map }

O Godot reconstrói o mapa inteiro por conta própria quando a cena deixa de combinar com o que uma construção completa daria. Quando o modo ao vivo
começa numa cena, ele faz isso na hora, senão assim que você faz uma pausa na edição:

| Quando | Por quê |
| --- | --- |
| O modo ao vivo começa numa cena construída com outras definições de entidades | A cena foi construída antes de você instalar o pacote de entidades de gameplay ou adicionar uma definição, então algumas entidades são nós simples |
| O modo ao vivo começa numa cena que edições ao vivo anteriores mudaram | A cena ainda guarda edições ao vivo de uma sessão anterior que você nunca salvou |
| As definições de entidades mudam enquanto o modo ao vivo roda | Você salvou um FGD ou uma definição no Inspetor do Godot, uma classe de script mudou, ou chegaram arquivos novos, como um pacote instalado |
| Você salva um prefab que o mapa usa | A cena ainda mostra o prefab como era |

Cenas abertas fora do modo ao vivo mantêm as entidades como foram construídas, e o painel Output do Godot avisa quando uma delas
foi construída com outras definições de entidades. Aperte **Build Map** nela e salve.

### Entidades sem definição { #entities-without-a-definition }

Quando o modo ao vivo constrói uma entidade cuja classe o Godot não tem definição, a barra de status nomeia a classe e o FGD em que
adicioná-la:

![A barra de status dizendo que o Godot não tem definição para crypt_lamp, ele mostra um nó simples, adicione a classe ao FGD de res://crypt_settings.tres](../assets/live/status-no-definition.png)

A dica do botão do modo ao vivo guarda a mesma nota enquanto o mapa usa essa classe:

![O botão do modo ao vivo ligado, a dica dizendo que as edições chegam ao Godot antes de você salvar, e que o Godot não tem definição para crypt_lamp](../assets/live/toolbar-live-mode-no-definition.png)

Adicione a definição e salve, e o Godot reconstrói o mapa por conta própria. A nota some no seu próximo salvamento. Veja
[Entidades que não fazem nada](entities-troubleshooting.md) para saber onde uma definição vai.

## Hot reload em um jogo em execução { #hot-reload-in-a-running-game }

Para ver seus salvamentos em um jogo que já está rodando:

1. Adicione o autoload

    No Godot, vá em *Project > Project Settings > Globals > Autoload* e adicione este script:

    ```
    res://addons/func_godot/src/trenchdot/runtime/trenchdot_hot_reload.gd
    ```

2. Rode o jogo pelo editor do Godot

    O autoload só escuta em jogos iniciados pelo editor do Godot, a menos que você defina o `enabled_in_exported_builds` dele
    como true.

3. Salve no TrenchDot

    O jogo reconstrói os mapas que usam o arquivo salvo e emite `map_reloaded(map)`. Use esse sinal para pôr o jogador
    de volta num ponto de spawn ou redefinir o estado do jogo. A barra de status deve terminar com "o jogo em execução recarregou 1 mapa(s)".

Ele escuta na porta 7843, que precisa combinar com *Recarga a quente do jogo em execução* nas preferências do editor. O TrenchDot só envia
salvamentos para ele enquanto o Live Link está ligado.
