---
source: ac40f8ef73c25c546d11381f2a654f03d847b7aa
---

# Scripts MCP { #mcp-scripts }

Um script MCP é um arquivo JSON com uma lista de chamadas de ferramentas do [MCP](mcp.md), e o `run_script` as reproduz em ordem. Ele guarda como um mapa foi construído em passos legíveis, então você pode reconstruir o mapa do zero e revisá-lo como texto num pull request, e os testes também podem reproduzi-lo.

Os mapas de showcase em [examples/mcp](https://github.com/Paraxdev/TrenchDot/tree/main/examples/mcp) são construídos assim, e servem também de exemplo para cada ferramenta.

| Script | Constrói |
| --- | --- |
| `mountain_house.json` | Uma cabana pendurada num penhasco, com um elevador de penhasco e florestas |
| `church_school.json` | Uma igreja e uma escola em que três botões abrem o portão do pátio |
| `lighthouse_forest.json` | Um farol com um sinalizador giratório, um barco num `func_train` e fogos-fátuos que surgem |
| `withered_city.json` | Uma cidade neon em ruínas à noite, com relâmpagos, brilho e `ssr` |
| `night_district.json` | Uma rua industrial à noite com um portão de enrolar e um trem de viaduto |
| `sea_island.json` | O [tutorial da ilha no mar](tutorials/sea-island.md) |
| `scripted_scene.json` | O [tutorial da cutscene](tutorials/cutscene.md) |
| `logic_playground.json` | Um salão para os [nós de matemática e lógica](gameplay/entities/logic.md): moedas, uma pontuação, um portão na porta do cofre e uma loteria de luzes |

## Executando { #running-them }

Inicie o editor com o projeto de demonstração e o servidor HTTP dele, veja [Iniciando](mcp.md#starting-it), e depois reproduza um script com o `tools/mcp_script.py`. Se o editor pedir um [token](mcp.md#the-http-token), passe-o com `--token` ou `TRENCHDOT_MCP_TOKEN`. O comando `shot` salva uma captura de tela a partir da posição de câmera que você der, e `--shade` escolhe o sombreamento:

```sh
trenchdot --mcp-http --project godot                        # the editor with the demo project
python tools/mcp_script.py run examples/mcp/sea_island.json   # replay a script
python tools/mcp_script.py shot view.png --pos 4300,900,3300 --look 2200,700,1200 --shade lit
```

Os scripts salvam em `godot/demo/maps/showcase`, exceto `scripted_scene.json` e `logic_playground.json`, que salvam em `godot/demo/maps`.

Uma execução para no primeiro passo que falha, a menos que você passe `--continue-on-error`, e `--url` aponta o executor para um editor em outra porta. Cada execução é um passo de desfazer em cada mapa que ela altera, com o nome do `label` do script ou do arquivo dele.

## Formato do script { #script-format }

```json
{
  "format": "trenchdot-mcp-script",
  "label": "Hollow room",
  "vars": { "save_dir": "${project}/maps" },
  "steps": [
    { "note": "A hollow room." },
    { "tool": "map_file", "args": { "op": "new" } },
    { "tool": "create_brush", "args": { "min": [0, 0, 0], "max": [256, 128, 256] }, "save": "room" },
    { "tool": "select", "args": { "ids": "$room.ids" } },
    { "tool": "run_action", "args": { "action": "csg_hollow" } },
    { "tool": "map_file", "args": { "op": "save", "path": "${save_dir}/room.tdm" } }
  ]
}
```

`vars` define valores padrão para as suas próprias variáveis. Um objeto `vars` passado ao `run_script` os substitui, então um mesmo script pode salvar em lugares diferentes, e as pastas que você passa desse jeito são pastas de onde o script pode ler e em que pode gravar.

Um passo sem `tool` é um comentário. Um passo com `save` guarda o resultado sob esse nome, para os passos seguintes poderem usar os ids que ele criou.

Scripts são compartilhados como mapas, então os passos deles só editam e salvam o mapa. Um passo não pode abrir outro projeto nem um dos mapas dele, iniciar o Godot, construir nele, ligar o modo ao vivo, desligar o MCP, copiar ou recortar para a área de transferência, descartar alterações não salvas nem fechar as outras abas. Desfazer e refazer só alcançam o que o próprio script fez, então o trabalho que você ainda não salvou continua em todas as abas. `screenshot`, `simulate_input`, `walkability` e `run_script` também não podem rodar dentro de um script, nem as instalações de `project_content` que não sejam `gameplay`.

Todo arquivo que um passo lê ou grava precisa estar no projeto aberto, na pasta do próprio script ou numa pasta que você passa em `vars`, mesmo quando um link dentro deles aponta para outro lugar. Isso cobre os mapas que ele abre, verifica, compara, salva e exporta, os mapas, modelos, prefabs e texturas que ele importa com os arquivos que eles nomeiam, e tudo o que as conversões, importações, exportações e a biblioteca de prefabs gravam. Prefabs, arquivos de modelo, texturas e instâncias do Hammer que um mapa ou modelo nomeia em outro lugar ficam de fora, caminhos de rede são recusados antes de o editor olhar para eles, e um salvamento sem caminho só grava de volta no mapa que estava aberto quando o script começou.

A regra vale também para arquivos de material, suas imagens e arquivos de hotspot, a biblioteca de prefabs e os presets de natureza. Uma textura cujos arquivos ficam fora dessas pastas conta como ausente dentro de um script, então ajustá-la ou alinhá-la volta ao tamanho de textura padrão. O resumo então lista os arquivos que ele não conseguiu ler em `unread_textures`.

Um passo recusado aparece nos `errors` do resumo, com o motivo. O resumo conta as chamadas de ferramenta como `steps` e os comentários como `notes`.

As strings em `args` podem se referir a resultados salvos e a algumas variáveis embutidas:

| Escreva | Significa |
| --- | --- |
| `"$name.path"` | O valor JSON salvo. Números indexam arrays, como em `$room.ids.0` |
| `"${name.path}"` | O texto dele dentro de uma string mais longa |
| `"$a.ids + $b.ids"` | Os dois juntos num único array. `["$a.ids", "$b.ids"]` também funciona |
| `$last` | O resultado do passo anterior |
| `$project` | A pasta do projeto Godot aberto, um erro quando nenhum está aberto |
| `$script_dir` | A pasta do arquivo do script |
| `$$` | Um `$` literal, então um caminho de nó é `"$$Player"` |

Um nome desconhecido é um erro. Um `$` que não pode iniciar um nome, como em `"$5"`, fica como está. Ids dados como texto, como `"42"`, funcionam onde quer que se espere um id de nó.

`$name/rest` sem chaves não se expande, então um `$project/maps` sem chaves é um erro em vez de um caminho que ninguém quis escrever. Use `"${project}/maps"`.

## Ids depois do CSG { #ids-after-csg }

Os passos de CSG e de clip substituem os brushes que cortam por outros com ids novos. O executor reescreve os ids salvos por passos anteriores para acompanhar, então `"$wall.ids"` continua significando a parede inteira depois que você corta uma janela nela, e um brush removido sai da lista.

Só os valores sob chaves que carregam ids de nó são reescritos: `id`, `ids`, `selection`, `selected`, `nodes`, `letters`, `copies`, `entity`, `group`, `parent`, `insert_parent`, `current_layer`, `open_groups`, `targets` e chaves que terminam em `_id` ou `_ids`. Contagens como `moved` de `hierarchy reparent` ficam como estão, use o `ids` dele no lugar.

> [!TIP]
> Escreva por extenso toda configuração de brush nas chamadas `scatter` e `blend`. Tudo o que ficar de fora vem das configurações atuais do painel Scatter ou da ferramenta Blend do editor.
