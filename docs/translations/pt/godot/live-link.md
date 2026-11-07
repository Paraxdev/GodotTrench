---
source: c998bca32c1c4bd85f0c95739d78a13d4ede986a
---

# Protocolo do Live Link { #live-link-protocol }

O Live Link é como o TrenchDot e o editor do Godot conversam. O addon roda um pequeno servidor dentro do editor do Godot, e o
TrenchDot se conecta a ele para disparar reconstruções quando você salva, transmitir as edições do [modo ao vivo](live.md) e
mostrar se o Godot está rodando.

Você raramente vai precisar desta página, o link simplesmente funciona depois que o addon está ativado. Ela está aqui para quando
você escrever a sua própria ferramenta que controla o Godot do mesmo jeito, ou quiser descobrir por que a conexão falha. Se um
salvamento não reconstrói, comece por [Reconstruir ao salvar](live.md#rebuild-on-save), a barra de status costuma dizer o motivo.

## Transporte { #transport }

JSON delimitado por quebras de linha sobre TCP em `127.0.0.1:7842`. Cada mensagem tem um `event` e um `seq` opcional, e recebe
exatamente uma linha de resposta que repete o `seq` e traz `ok`. Falhas respondem `ok: false` com um `error`. O editor mantém uma
conexão aberta e envia um sinal de vida `status` a cada segundo.

Para usar outra porta, mude nos dois lados: `trenchdot/live_link_port` nas configurações do projeto do Godot e *Live Link do
Godot* em *Arquivo > Preferências* no TrenchDot.

O autoload de [hot reload](live.md#hot-reload-in-a-running-game) em um jogo em execução escuta em `127.0.0.1:7843` e só responde
`map_saved`.

## Eventos { #events }

Um cliente envia estes eventos. A coluna Resposta lista os campos com que o Godot responde além de `ok` e `seq`, ou o que ele
faz.

| Evento | Campos | Resposta |
| --- | --- | --- |
| `hello` | | `project` e `godot` (versão), uma checagem rápida de que o link responde |
| `status` | | `project`, `godot` (versão), `pid`, `scene` (a aba de cena que o Godot mostra) e `maps: [{path, epoch}]` dela |
| `map_saved` | `path` | `rebuilt`, o número de nós de mapa reconstruídos a partir do arquivo, `scene`, `waiting`, e os `missing_classes`, `settings` e `warnings` da construção |
| `build` | `path`, `text` | Como `map_saved`, construído a partir de `text` em vez do arquivo |
| `focus` | | Traz a janela do Godot para a frente |
| `export_game_config` | | Escreve `trenchdot_game.json` |
| `scan` | | Inicia uma varredura dos arquivos do projeto. Quando termina, cada sessão ao vivo confere as suas definições de entidades |
| `play` | | Salva as cenas abertas e executa a cena principal, ou `ok: false` com um `error` quando o projeto não tem uma |
| `live_begin` | `path`, `text`, `content` | `epoch` e `unresolved` |
| `live_resync` | `path`, `text` | `epoch` e `unresolved` |
| `live_delta` | `path`, `ops` | `epoch` e `unresolved`, ou `ok: false, resync: true` quando as ops não se encaixam |
| `live_end` | `path`, `revert` | Com `revert`, reconstrói a partir do arquivo salvo |
| `inspect` | `path`, `ids` | `nodes` com nome, classe, número de filhos e posição por id de nó do mapa, mais `live` e `pending` |
| `capture` | `path`, `camera`, `width`, `height`, `text` | `png` (base64), `width`, `height` e `warnings`, os erros e avisos registrados durante a construção |
| `walkable` | `path`, `agent`, `text` | `vertices` (lista plana de x, y, z), `polygons`, `islands` (índices de polígonos, o maior primeiro), `units_per_meter`, `agent` e `msec` |

`capture` renderiza o mundo do primeiro nó de mapa que usa `path`. `camera` traz `position` e `forward` nas unidades e eixos do
mapa e `fov`, o campo de visão horizontal em graus. Com `text`, o Godot constrói esse mapa antes, senão espera uma sessão ao vivo
alcançar. A resposta chega depois de uns doze quadros, e as outras mensagens são respondidas nesse meio tempo.

`walkable` calcula a malha de navegação da cena editada inteira com `TrenchDotNav` para `agent`
(`radius`, `height`, `max_climb`, `max_slope` em metros e graus) e a devolve nas unidades e eixos do mapa. Com `text`, o Godot
constrói esse mapa antes, a menos que ele tenha sido construído pela última vez a partir do mesmo texto.

Os caminhos são absolutos, com barras normais. Só participam os nós `FuncGodotMap` da cena editada, a aba de cena que o Godot
mostra, com *Auto Rebuild On Save* ligado. `scene` é o caminho `res://` dela, ou vazio quando não há cena aberta.

`waiting` lista as outras abas de cena abertas que usam o mapa. Depois de um `map_saved` elas constroem o arquivo quando a aba é
mostrada, depois de um `build` não. Um `map_saved` de um prefab também reconstrói os mapas da cena editada que o instanciam,
direto ou por meio de outros prefabs, e os conta em `rebuilt`.

O que as construções de um `map_saved` ou `build` relataram vem em três listas, cada uma só quando não está vazia:
`missing_classes` traz os nomes de classe ordenados que nenhuma definição descrevia, `settings` os caminhos das configurações do
mapa que as construções usaram, e `warnings` até 20 avisos de uma linha do relatório da construção. `unresolved` em uma resposta
ao vivo lista os nomes de classe ordenados que aquele passo construiu sem uma definição, também só quando existem.

`scan` serve para arquivos escritos fora do Godot, como um pacote de entidades instalado. Uma sessão ao vivo cuja cena foi
construída com definições de entidades diferentes das que as configurações do mapa têm depois da varredura constrói o mapa de novo.
Um addon sem `scan` ou `play` responde a eles como um evento desconhecido.

`text` é o mapa inteiro em JSON, com a iluminação pré-calculada em `lightmap` quando o mapa tem um bake, para que uma construção ao
vivo a mantenha. Isso soma cerca de 11 bytes por texel do atlas, uns 11 MB para um atlas de 1024 por 1024, então o editor só envia o
mapa inteiro de novo quando uma sessão começa, depois de um bake novo ou depois de uma mudança grande.

`content` é o id de conteúdo no bloco END do arquivo salvo, então uma cena construída a partir de um arquivo que não mudou não é
construída de novo quando uma sessão começa.

Quando a porta está ocupada, em geral por um segundo editor do Godot com o mesmo projeto, o addon avisa uma vez no painel Output e
tenta a porta de novo a cada três segundos, então ele assume quando o outro editor fecha.

## Ops ao vivo { #live-ops }

| Op | Campos | Significado |
| --- | --- | --- |
| `set` | `id`, `parent`, `index`, `node` | Insere ou substitui um nó, no formato JSON do formato `.tdm`, sem filhos |
| `remove` | `id` | Remove um nó com a sua subárvore, só o id removido mais acima é enviado |
| `translate` | `ids`, `offset` | Move nós, enviado durante o arrasto em vez da geometria |
| `properties` | `properties` | Substitui as propriedades do worldspawn |

O `epoch` de um mapa sobe sempre que ele é construído fora de uma sessão ao vivo ou os nós `FuncGodotMap` que o usam mudam. Quando
o próximo `status` mostrar um epoch novo, recomece com `live_begin`.

Os nós gerados levam o id de nó do mapa no metadado `_td_id`, e os nós de camada e de grupo em `_td_group`.

## Testando sem janela { #testing-without-a-window }

O link também funciona com um editor do Godot headless, o que serve para testes automatizados. Abra uma cena com um `FuncGodotMap`
nela e depois envie `inspect` para a porta 7842 para ver o que o Godot construiu:

```sh
godot --headless --editor --path godot res://path/scene.tscn
```

(Nota: em um checkout novo, rode `godot --headless --path godot --import` duas vezes antes. Uma passada só registra as classes do
addon.)

Um Godot headless não desenha nada, então `capture` responde com um erro nele. Teste as capturas com um editor com janela.

Os testes do addon fazem isso em
[`godot/tests/editor/live_link_tabs.gd`](https://github.com/Paraxdev/TrenchDot/blob/main/godot/tests/editor/live_link_tabs.gd),
que abre, fecha, recarrega e troca abas de cena em um editor headless e confere o que o link relata.

<div class="td-mcp" markdown>

## MCP { #mcp }

O [servidor MCP](../mcp.md) deixa agentes e scripts controlarem o editor, e algumas das suas ferramentas passam pelo Live Link.
A ferramenta `screenshot` usa `capture` quando se pede uma imagem do Godot.

Para testes headless que também editam o mapa, inicie o TrenchDot com o servidor MCP no mesmo projeto do editor do Godot
headless e edite o mapa por MCP:

```sh
trenchdot --mcp-http --project godot
```

</div>
