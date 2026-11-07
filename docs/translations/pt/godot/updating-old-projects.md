---
source: 4c71c99f5fabec89f69f0c99554b4f3a60fb4148
---

# Atualizando um projeto GodotTrench { #updating-a-godottrench-project }

O TrenchDot se chamava GodotTrench, e as builds beta se chamaram brevemente LevelBench. Se você configurou um projeto ou
salvou mapas naquela época, eles continuam funcionando. Você só atualiza o addon uma vez.

## Passos { #steps }

1. Inicie o editor novo. Na primeira inicialização ele copia suas preferências, projetos confiáveis e arquivos recentes da
   pasta de configurações antiga. A pasta antiga fica onde está, então o editor antigo também continua funcionando.
2. Abra o seu projeto do Godot. A janela de configuração abre e diz que o addon do projeto é mais antigo que o editor, mesmo
   quando os números de versão batem. Feche o Godot primeiro, depois aperte **Atualizar o addon para** seguido da versão.
3. Abra o projeto no Godot uma vez. O addon move as configurações de projeto dele de `godottrench/` para `trenchdot/` e escreve
   `trenchdot_game.json` no lugar de `godottrench_game.json`.

## O que se mantém { #what-carries-over }

| Você tinha | O que acontece |
| --- | --- |
| Mapas `.gtm` | Eles abrem como estão. A linha de status diz "room.gtm atualizado do formato GodotTrench, salve para mantê-lo". Salvar mantém o nome `.gtm`, então suas cenas ainda acham o mapa. Mapas novos são salvos como `.tdm`. |
| Variáveis de ambiente `GODOTTRENCH_` | Elas ainda funcionam. Quando os dois nomes estão definidos, o `TRENCHDOT_` vence. |
| O pacote de gameplay em `res://godottrench/entities` | Ele continua funcionando, e os seus próprios scripts que estendem as classes dele ainda compilam. Se você também instalar o pacote novo, o novo vence onde os dois definem uma entidade. Deixe a pasta antiga no lugar enquanto as suas cenas a usarem. |
| Pacotes de natureza e modelos em `res://godottrench/` | Os mapas que os posicionam continuam funcionando. Os presets de scatter procuram em `res://trenchdot/nature`, então adicione o pacote de natureza de novo para pintar scatter novo. |
| Cenas construídas pelo addon antigo | Elas carregam, porque o Godot acha os scripts do addon movido pelo uid. Construir o mapa de novo as atualiza. |

<div class="td-mcp" markdown>

## Clientes MCP { #mcp-clients }

O servidor MCP agora se chama `trenchdot`. Renomeie-o na configuração do seu cliente. A variável do token é
`TRENCHDOT_MCP_TOKEN`, e a antiga `GODOTTRENCH_MCP_TOKEN` ainda funciona.

</div>
