---
source: 5b0744accf923762bf8a9e744e70cb250be2483b
---

# Linha de comando { #command-line }

```sh
trenchdot [options] [map]
```

Sem opções o editor só abre, com o mapa se você der um. `trenchdot --help` (ou `-h`) lista as opções
e sai.

| Opção | O que faz |
| --- | --- |
| `--project <dir>` | Carrega o projeto Godot em `<dir>` ou numa pasta acima dele e confia nele durante a sessão, veja [Confiança no projeto](godot/running-godot.md#project-trust). Sem ela o editor carrega o último projeto |
| `--default-prefs` | Começa com as preferências padrão e nunca as salva. Útil para scripts e testes, e para conferir se um problema vem das suas próprias configurações |
| `--offscreen` | Abre a janela fora de todas as telas, sem foco nem botão na barra de tarefas. Ela ainda renderiza, então scripts e testes podem tirar capturas de tela enquanto você continua trabalhando |
| `--live-link-port=<port>` | Fala com o editor do Godot nesta porta em vez da porta de *Live Link do Godot* nas Preferências, só nesta execução. Scripts e testes passam uma porta livre, então os saves deles nunca reconstroem mapas no Godot que você tem aberto |
| `--hot-reload-port=<port>` | Faz o mesmo com a porta de *Recarga a quente do jogo em execução*, na qual um jogo em execução escuta para recarregar mapas salvos |
| `--dump <map>` | Imprime um mapa como JSON e sai |
| `--to-json <map> <out>` | Escreve um mapa como JSON e sai |
| `--to-tdm <map> <out>` | Escreve um mapa como `.tdm` binário e sai |
| `--export-glb <map> <out.glb>` | Escreve a geometria e os materiais do mapa como glTF binário para o Blender e outras ferramentas 3D, e sai |
| `--export-obj <map> <out.obj>` | Escreve o mesmo como OBJ, com um `.mtl` e uma pasta de texturas ao lado, e sai |

Uma opção desconhecida, ou uma sem o seu valor, imprime um erro e sai com o código 2, nenhuma janela abre. Uma conversão ou
exportação que falha imprime o motivo e sai com o código 1.

As conversões e exportações nunca abrem uma janela, então são seguras em scripts e hooks do git, veja
[Arquivos de mapa no git](development.md#map-files-in-git).

As exportações acham o projeto Godot do mapa a partir da pasta dele e usam os padrões de *Arquivo > Exportar*: o mapa inteiro,
sem objetos escondidos nem instâncias de scatter. A lógica de entidades, os triggers e os scripts ficam de fora, veja
[Exportando para o Blender e outras ferramentas 3D](editor/exporting.md).

<div class="td-mcp" markdown>

## MCP { #mcp }

Estas opções preparam o [servidor MCP](mcp.md), para scripts e agentes de IA controlarem o editor.

| Opção | O que faz |
| --- | --- |
| `--mcp` | Serve MCP por stdin e stdout |
| `--mcp-http[=<port>]` | Serve MCP por HTTP em 127.0.0.1, porta 7841 por padrão |
| `--mcp-token=<token>` | Faz os clientes HTTP enviarem este token, seja o que for que as Preferências dizem. Ele vale mais que `TRENCHDOT_MCP_TOKEN`, veja [O token HTTP](mcp.md#the-http-token) |

`--default-prefs` combina bem com qualquer uma, para uma sessão com script começar sempre das mesmas configurações. Com
`TRENCHDOT_DISABLE_MCP=1` definido, nenhuma das duas opções inicia um servidor, veja [Desligando o MCP](mcp.md#turning-mcp-off).

</div>
