---
source: d5a662c31724dba93b77bd318ab3a570c43e95b6
---

# Servidor MCP { #mcp-server }

O editor tem um servidor [MCP](https://modelcontextprotocol.io) embutido. Um agente como o Claude Code, ou um simples script em Python,
pode usá-lo para inspecionar e editar mapas, tirar capturas de tela e enviar entrada para a interface.

Use-o para deixar um assistente de IA montar ou revisar um nível, para automatizar edições repetitivas, ou para testar o editor
automaticamente. O servidor fica desligado até você iniciá-lo, e só escuta na sua própria máquina.

## Iniciando { #starting-it }

| Transporte | Como iniciar |
| --- | --- |
| HTTP | Abra o editor com `--mcp-http` para a porta 7841, ou `--mcp-http=PORT` para outra porta. Para ele iniciar sempre, vá em *Arquivo > Preferências*, abra *Avançado*, marque *ativar ao iniciar* ao lado de *Servidor MCP (HTTP, localhost)*, defina a porta ao lado e reinicie |
| stdio | Abra com `--mcp`. O cliente inicia o próprio editor a cada sessão e conversa com ele por entrada e saída padrão |

Escolha HTTP quando você mantém o editor aberto e trabalha nele enquanto um agente ajuda. Escolha stdio quando o cliente
deve ter o seu próprio editor, ou quando outras pessoas usam o mesmo computador. Para conectar o Claude Code a qualquer um dos dois:

```sh
claude mcp add --transport http trenchdot http://127.0.0.1:7841/mcp   # HTTP
claude mcp add trenchdot -- /path/to/trenchdot --mcp                 # stdio
```

O `.mcp.json` do repositório já aponta o Claude Code para o servidor HTTP. *Arquivo > Preferências* mostra o comando
do servidor que está rodando, e *Copiar comando* o coloca na área de transferência.

Iniciou o editor em outra porta? Defina `TRENCHDOT_MCP_URL` e
[tools/mcp_client.py](https://github.com/Paraxdev/TrenchDot/blob/main/tools/mcp_client.py) e
[tools/mcp_script.py](https://github.com/Paraxdev/TrenchDot/blob/main/tools/mcp_script.py) se conectam ali.

### O token HTTP { #the-http-token }

Qualquer programa no seu computador alcança o servidor HTTP, e outras pessoas com sessão aberta nele também. Para mantê-las fora,
marque *Exigir um token* em *Avançado* em *Arquivo > Preferências*. A partir da próxima requisição, o servidor só responde a
requisições que enviam o token dele como `Authorization: Bearer <token>`. O comando nas Preferências passa a incluir o token,
use-o no lugar do `.mcp.json`. O stdio não precisa de token, só o cliente que iniciou o editor pode falar com ele.

O editor cria um token aleatório na primeira vez que você pede um e o guarda em `mcp-token`, um arquivo que só você pode ler,
ao lado das suas configurações: `%APPDATA%\TrenchDot\data` no Windows, `~/.local/share/trenchdot` no Linux e
`~/Library/Application Support/TrenchDot` no macOS. Uma execução com `--default-prefs` guarda o seu próprio em
`trenchdot-default-prefs.mcp-token` na pasta temporária.

Para escolher o token você mesmo em uma execução, defina `TRENCHDOT_MCP_TOKEN` ou abra com `--mcp-token=<token>`, que vale mais
que a variável. Qualquer um dos dois liga o token seja o que for que as Preferências dizem, e *Exigir um token* aparece marcado e
esmaecido. `tools/mcp_client.py` e `tools/mcp_script.py` só enviam um token quando você dá um, com `--token` ou
`TRENCHDOT_MCP_TOKEN`.

## Desligando o MCP { #turning-mcp-off }

Para manter agentes e scripts fora do editor por completo, desmarque *MCP* no topo da parte de MCP de
*Arquivo > Preferências*. O servidor HTTP fecha na hora e derruba as conexões abertas, um cliente em stdio recebe um erro
a cada requisição, e as outras configurações de MCP ficam esmaecidas. A configuração é salva na hora, então todo editor que você iniciar
sem `--default-prefs` mantém o MCP desligado até você marcá-lo de novo, o que traz o servidor HTTP de volta na sua porta.

![A parte de MCP das Preferências: MCP marcado, a linha do servidor HTTP com a porta dele, e Exigir um token](assets/mcp/prefs-mcp.png)

Para desligar o MCP em uma execução seja o que for que as Preferências dizem, defina `TRENCHDOT_DISABLE_MCP` com qualquer valor que não seja vazio ou `0`.
*MCP* aparece então desmarcado e esmaecido. Com o MCP desligado, `--mcp` e `--mcp-http` não iniciam servidor. O editor ainda
abre e a linha de status diz que o MCP está desligado, e um cliente em stdio recebe um erro dizendo isso em vez de esperar.

Um agente também pode desligar o MCP, com `set_editor` e `"mcp": false`. Só você pode ligá-lo de novo.

## Ferramentas { #tools }

| Área | Para que serve | Ferramentas |
| --- | --- | --- |
| Inspecionar | Ler o mapa, o estado do editor, as definições de entidade e como usar uma entidade a partir do código do jogo, sem mudar nada | `get_state`, `summarize_map`, `list_nodes`, `get_node`, `changes_since`, `validate_map`, `get_game_config`, `code_reference` |
| Criar | Adicionar brushes, malhas, terrenos, entidades e modelos importados, e fazer trim ao longo das bordas das faces | `create_brush`, `create_mesh`, `create_terrain`, `create_entity`, `import_model`, `trim` |
| Editar | Mudar o que existe: chaves de entidade, seleção, posicionamento, faces, texturas, terreno, camadas e grupos, e worldspawn | `update_entity`, `select`, `transform`, `duplicate`, `set_face`, `mesh_edit`, `texture`, `terrain_edit`, `hierarchy`, `set_map_properties`, `bake_lighting` |
| Pintura e gameplay | Espalhar modelos, misturar materiais, colar peças da biblioteca de prefabs, e transformar brushes em portas, plataformas e botões | `scatter`, `blend`, `prefab_library`, `random_fill`, `gameplay` |
| Editor | Rodar ações de menu, abrir e salvar arquivos, mudar configurações do editor, mover a câmera, tirar capturas de tela, enviar entrada e conferir por onde um jogador pode andar | `run_action`, `map_file`, `open_project`, `set_editor`, `set_camera`, `screenshot`, `simulate_input`, `walkability` |
| Projeto | Adicionar o pacote de entidades de gameplay, os modelos de natureza ou a demo ao projeto Godot aberto | `project_content` |
| Scripts | Reproduzir uma lista salva de chamadas de ferramenta, veja [Scripts MCP](mcp-scripts.md) | `run_script` |

A descrição de cada ferramenta lista as operações e os argumentos dela, então um agente aprende tudo direto do servidor.
[tools/mcp_client.py](https://github.com/Paraxdev/TrenchDot/blob/main/tools/mcp_client.py) as imprime com `list`
e chama uma ferramenta pelo shell:

```sh
python tools/mcp_client.py call screenshot '{"target":"3d","width":1280,"height":720}' --out shot.png
```

## Capturas de tela { #screenshots }

`screenshot` devolve um PNG da janela inteira do editor, ou de uma vista quando `target` é `3d`, `top`, `front` ou `side`.
Para uma vista, os outros argumentos decidem o que você recebe:

| Argumentos | Resultado |
| --- | --- |
| Nenhum | A janela inteira do editor |
| Só `target` | Aquela vista acoplada como está, com os overlays |
| `width` e `height` | Renderizada fora da tela até 4096 pixels, sem grade, caixas de entidade, gizmos nem seleção |
| `width`, `height` e `"overlays": true` | Fora da tela, com os overlays do editor |
| `"overlays": false` | Uma captura limpa no tamanho acoplado |

### O que o Godot renderiza { #what-godot-renders }

`"source": "godot"` pede ao editor do Godot para renderizar o mapa com as luzes, o ambiente, a névoa e o pós-processamento
de verdade. Você recebe esse PNG de volta, mais os erros ou avisos que o Godot registrou ao construí-lo. Útil para julgar
iluminação e materiais sem abrir o jogo.

```sh
python tools/mcp_client.py call screenshot '{"source":"godot","position":[0,256,512],"look_at":[0,64,0]}' --out godot.png
```

1. Salve o mapa dentro do projeto que está aberto no TrenchDot.
2. Abra esse projeto no editor do Godot, com uma cena cujo `FuncGodotMap` constrói o mapa.
3. Chame `screenshot` com `"source": "godot"`. Sem `position` e `look_at` ele usa a câmera da vista 3D.

Sem o [modo ao vivo](godot/live.md) o Godot primeiro constrói o mapa como mostrado no TrenchDot, como *Construir no Godot*.
`"build": false` captura a cena como o Godot a tem. `width` e `height` padronizam em 1280 por 720, e `fov` no da vista
3D.

> [!NOTE]
> O Godot precisa de uma janela para renderizar, um Godot headless devolve um erro em vez de uma imagem. A janela pode ficar
> em segundo plano ou minimizada.

## Lendo e revisando um mapa { #reading-and-reviewing-a-map }

| Passo | Ferramenta |
| --- | --- |
| Ter uma visão geral | `summarize_map` lista as camadas, as classes de entidade, as conexões de I/O e os materiais, e os assets que faltam |
| Achar nós | `list_nodes` filtra por `layer`, `material`, `property` (`{"model": "*crate*"}`) ou uma `box` no mapa |
| Olhar um | `get_node` devolve um nó. `compact: true` deixa de fora vértices e faces |
| Mostrar o que você mudou | `changes_since` compara com o último salvamento, com um número de `undo_steps` atrás, ou com um `file` de mapa |
| Desfazer | Cada chamada e cada `run_script` é um passo de desfazer chamado `MCP: ...`. `run_action undo` com `steps` desfaz vários |

Veja [Revisando alterações](editor/reviewing.md) para manter mapas no git.

## Comportamento que vale conhecer { #behavior-worth-knowing }

| Assunto | Comportamento |
| --- | --- |
| Navegadores | O servidor HTTP recusa qualquer requisição com um cabeçalho `Origin` e qualquer POST que não seja `application/json`, então páginas da web não o alcançam, nem as servidas do localhost |
| Erros | Uma ferramenta que não conseguiu fazer o trabalho devolve `isError`, inclusive um `run_action` que falhou |
| Diálogos de arquivo | Ações que abrem um são recusadas, use `map_file` com um caminho. `run_action save` precisa de um mapa que já tenha arquivo |
| Abas não salvas | `run_action close_tab` recusa alterações não salvas, a menos que `args.discard` seja true. `args.close_others` fecha todas as abas menos a ativa |
| Caminhos de arquivo | `res://` e caminhos relativos em `map_file`, `run_script`, `import_model`, `changes_since` e `validate_map` partem do projeto aberto, ou do diretório de trabalho do editor sem um. `map_file open` e `open_tab` resolvem o caminho para um absoluto antes de abrir, então um `save` posterior não pode cair em outro lugar |
| Configurações do editor | `transform` e `run_action rotate` e `flip` giram em torno do centro da seleção, encaixado na grade só enquanto o encaixe está ligado, e mantêm as texturas travadas enquanto o bloqueio de UV está ligado. `transform` aceita `snap` e `uv_lock` para mudar isso em uma chamada, `duplicate` aceita `uv_lock`. `run_action sculpt` e `blend` usam os ajustes de ferramenta que uma pessoa pode mudar, `terrain_edit` usa padrões fixos |
| Seleção | `select` recusa ids desconhecidos e pula nós escondidos ou travados, listando-os em `skipped` |
| CSG | `csg_subtract`, `csg_merge`, `csg_intersect`, `csg_hollow` e `clip_apply` devolvem `replaced`, os ids antigos mapeados para os novos |
| Tempos limite | As chamadas expiram em 120 s, exceto `run_script`, `project_content` e `bake_lighting` |
| Pergunta de conteúdo | A pergunta que um projeto novo recebe sobre conteúdo pronto se fecha na primeira chamada que não seja `get_state`, `screenshot` e `simulate_input`, e não abre de novo nessa sessão |
| Variáveis de script | `$name/rest` (sem chaves) é um erro quando `name` é uma variável conhecida, já que só `$name` sozinho ou `${name}/rest` a expandem. Escreva `${project}/maps` |

## Notas das ferramentas { #tool-notes }

| Ferramenta | Nota |
| --- | --- |
| `run_action` `copy`, `cut`, `paste` | Copy e cut devolvem o texto da área de transferência, paste o recebe como `args.text` com um `offset` ou `origin` opcional |
| `run_action` `clip_apply` | Recebe um plano (`point` e `normal`, ou três `points`) e `keep`: `front`, `back` ou `both` |
| `run_action` `create_decal` | Coloca um [decal](editor/decals.md), use-o no lugar de brushes finos |
| `create_brush` | `openings` abre portas e janelas, `{min, max, count, step, rows, row_step}` repete uma como uma grade |
| `create_mesh` `rotate` | `[x, y, z]` graus na ordem YXZ do Godot, então `[0, 0, 90]` deita um cilindro ao longo de X |
| `transform` | Aplica translate, rotate, flip e scale_to nessa ordem, como um passo de desfazer. `rotate` gira 90 graus em torno de Y, a menos que se diga outra coisa |
| `create_terrain` | `origin` é o canto mínimo do terreno, ao contrário de outras ferramentas. `center` o coloca em volta de um ponto |
| `create_brush`, `create_mesh` | Uma cumeeira de duas águas corre ao longo de Z em brushes e ao longo de X em malhas. As duas ferramentas aceitam `ridge_x` e `ridge_z` |
| `map_file` exports | `path` é o arquivo escrito, mantido para chamadores antigos, `exported` também o nomeia e `map_path` é o mapa aberto |
| `terrain_edit` | `probe [x, z]` sem `op` só lê a altura. `clear_layer` precisa de `layer` |
| `validate_map` | Aceita qualquer método ou propriedade da classe Godot ou do script do alvo como entrada. `coplanar_faces` marca faces que vão piscar. Todo problema tem `bounds`. `path` confere um mapa salvo sem abri-lo, `project: true` confere todos os mapas do projeto |
| `run_action` `place_entities` | Recebe `args.classnames`, e `at` e `normal` para apoiar as entidades numa superfície como um arrasto da aba Entidades faz. Sem classnames é um erro |
| `simulate_input` drags | Um arrasto com `"release": false` mantém o botão e seus modificadores apertados depois que a chamada retorna, então uma captura de tela mostra o arrasto em andamento, um cartão ao lado do cursor por exemplo. Um evento `release` em uma chamada posterior o solta onde o ponteiro está, ou em `x` e `y`, ou `world`, e um release com só um de `x` e `y` é um erro. O arrasto e seus modificadores continuam apertados, inclusive no teclado de verdade, até esse release, uma ação `new_map` ou um `close_tab` bem-sucedido, ou o MCP ser desligado, então nunca esqueça o release. Cliques e arrastos são recusados até lá |
| `run_action` `reload_materials` | Reescaneia a pasta de texturas agora, embora arquivos novos sejam achados sozinhos de qualquer jeito |
| `set_editor` `live_link_port` | Fala com um editor do Godot cujo projeto define outro `trenchdot/live_link_port` |
| `scatter` `preset` | Adiciona os modelos embutidos do Blockbench que uma predefinição precisa. Uma predefinição glTF num projeto sem o pacote de natureza é um erro que nomeia `project_content` |
| `bake_lighting` | Pré-calcula como [Pré-calcular iluminação](editor/light-baking.md) e espera terminar. Opções omitidas mantêm o que o mapa teve no último cálculo. `bake: false` só troca a `view`, `remove: true` descarta o cálculo. `out_of_date` diz se o mapa mudou desde então |
| `project_content` | `install: "nature"` ou `"demo"` baixa da release e responde quando os arquivos estão no lugar. `install: "gameplay"` copia o pacote de entidades de gameplay do addon do projeto, inclusive dentro de scripts. Sem argumentos diz o que o projeto tem |
