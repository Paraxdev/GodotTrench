---
source: 92464837c8b6e8506e85d0270bb18896476cd41b
---

# Estrutura do JSON { #json-layout }

A forma JSON é o que os editores mais antigos salvavam. É também o que `--dump`, um salvamento em `.json`, a área de transferência e o Live Link usam.

Os leitores tratam qualquer arquivo que não comece com `\x89GTM` como JSON, com ou sem marca de ordem de bytes UTF-8. Um mapa JSON antigo carrega como está e passa a binário no próximo salvamento em `.tdm`.

O [`json_fmt.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/json_fmt.rs) o imprime de um jeito que mantém os diffs legíveis. Ele indenta com dois espaços, mantém as chaves na ordem em que estas páginas as listam e coloca arrays e objetos curtos numa linha só. Cada face ocupa uma linha, e um array `vertices` de até 64 entradas também, então mover um brush só muda as linhas desse brush.

A sua própria ferramenta não precisa copiar esse layout, qualquer JSON válido com o mesmo conteúdo carrega do mesmo jeito.

## Convertendo { #converting }

| Comando | Resultado |
| --- | --- |
| `trenchdot --dump map.tdm` | Imprime o mapa como JSON |
| `trenchdot --to-json map.tdm map.json` | Grava o mapa como JSON |
| `trenchdot --to-tdm map.json map.tdm` | Grava um mapa binário, recusando um arquivo que o editor não conseguiria abrir |

As conversões copiam o conteúdo do arquivo sem carregá-lo no editor, então mantêm chaves que o editor não conhece.

O editor também abre mapas `.json`, e *Salvar como* com um nome `.json` grava JSON. Para diffs em JSON de mapas binários no git, veja [Arquivos de mapa no git](../development.md#map-files-in-git).

## Área de transferência { #clipboard }

Copiar e colar usam a mesma forma de nó em outro invólucro:

```json
{
  "format": "trenchdot-clipboard",
  "version": 1,
  "nodes": [ ... ]
}
```

`nodes` guarda as subárvores copiadas sem os pais delas. Colar executa as mesmas verificações do carregamento, dá a cada nó um id novo e mantém `label`, `hidden` e `locked`. Uma layer copiada cola os seus filhos.

O Live Link envia nós avulsos na mesma forma, sem `children`.

## Exemplo { #example }

Um trecho reduzido de `godot/demo/maps/scripted_scene.tdm` como o `trenchdot --dump` o imprime, com o brush do chão e um barril explosivo:

```json
{
  "format": "trenchdot-map",
  "version": 1,
  "properties": {
    "ambient_color": "60 66 82",
    "classname": "worldspawn",
    "message": "Scripted Scene",
    "sky_ground_color": "60 62 70",
    "sky_horizon_color": "120 130 150",
    "sky_top_color": "40 54 92",
    "sun_angles": "-40 -55",
    "sun_energy": "0.6"
  },
  "layers": [
    {
      "id": 1,
      "type": "layer",
      "name": "Default",
      "color": "#7e67e6ff",
      "omit_from_export": false,
      "children": [
        {
          "id": 2,
          "type": "brush",
          "vertices": [[320.0,8.0,-320.0],[320.0,8.0,320.0],[320.0,0.0,320.0],[320.0,0.0,-320.0],[-320.0,8.0,320.0],[-320.0,8.0,-320.0],[-320.0,0.0,-320.0],[-320.0,0.0,320.0]],
          "faces": [
            {"indices":[0,1,2,3],"material":"showcase/cobble","uv":{"u_axis":[0.0,0.0,-1.0],"v_axis":[0.0,-1.0,0.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[4,5,6,7],"material":"showcase/cobble","uv":{"u_axis":[0.0,0.0,1.0],"v_axis":[0.0,-1.0,0.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[1,0,5,4],"material":"showcase/cobble","uv":{"u_axis":[1.0,0.0,0.0],"v_axis":[0.0,0.0,1.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[3,2,7,6],"material":"showcase/cobble","uv":{"u_axis":[1.0,0.0,0.0],"v_axis":[0.0,0.0,1.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[2,1,4,7],"material":"showcase/cobble","uv":{"u_axis":[1.0,0.0,0.0],"v_axis":[0.0,-1.0,0.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[6,5,0,3],"material":"showcase/cobble","uv":{"u_axis":[-1.0,0.0,0.0],"v_axis":[0.0,-1.0,0.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}}
          ]
        },
        {
          "id": 17,
          "type": "entity",
          "classname": "prop_physics",
          "origin": [0.0,24.0,176.0],
          "angles": [0.0,0.0,0.0],
          "properties": {
            "explosion_damage": "25",
            "explosion_radius": "256",
            "explosive": "1",
            "health": "5",
            "model": "res://demo/scenes/barrel.tscn",
            "size": "16 24 16",
            "targetname": "barrel"
          },
          "outputs": [{"output":"broken","target":"hp_readout","input":"run"}]
        }
      ]
    }
  ]
}
```

O chão é uma laje de 640 por 640 unidades com 8 unidades de espessura e o topo em Y 8. A primeira face dela, `[0,1,2,3]`, é o lado +X, com enrolamento anti-horário visto de +X.

<div class="td-mcp" markdown>

## MCP { #mcp }

As ferramentas do [servidor MCP](../mcp.md) também leem e devolvem nós nesta forma JSON.

</div>
