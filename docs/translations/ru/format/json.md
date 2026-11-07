---
source: 92464837c8b6e8506e85d0270bb18896476cd41b
---

# Формат JSON { #json-layout }

Форма JSON это то, что сохраняли старые редакторы. Её же используют `--dump`, сохранение в `.json`, буфер обмена и live
link.

Читатели считают JSON любой файл, который не начинается с `\x89GTM`, с меткой порядка байтов UTF-8 или без неё. Старая
карта JSON загружается как есть и становится бинарной при следующем сохранении в `.tdm`.

[`json_fmt.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/json_fmt.rs) печатает её так, чтобы
различия оставались читаемыми. Он делает отступ в два пробела, держит ключи в том порядке, в каком их перечисляют эти
страницы, а короткие массивы и объекты помещает в одну строку. Каждая грань занимает одну строку, как и массив
`vertices` до 64 записей, поэтому перемещение одного браша меняет только строки этого браша.

Вашему инструменту не обязательно копировать эту раскладку, любой корректный JSON с тем же содержимым загружается так же.

## Преобразование { #converting }

| Команда | Результат |
| --- | --- |
| `trenchdot --dump map.tdm` | Печатает карту как JSON |
| `trenchdot --to-json map.tdm map.json` | Записывает карту как JSON |
| `trenchdot --to-tdm map.json map.tdm` | Записывает бинарную карту, отклоняя файл, который редактор не смог бы открыть |

Преобразования копируют содержимое файла, не загружая его в редактор, поэтому сохраняют ключи, которых редактор не знает.

Редактор открывает и карты `.json`, а *Сохранить как* с именем `.json` записывает JSON. О различиях JSON для бинарных карт
в git см. [Файлы карт в git](../development.md#map-files-in-git).

## Буфер обмена { #clipboard }

Копирование и вставка используют ту же форму узла в другой обёртке:

```json
{
  "format": "trenchdot-clipboard",
  "version": 1,
  "nodes": [ ... ]
}
```

`nodes` хранит скопированные поддеревья без их родителей. Вставка выполняет те же проверки, что и загрузка, даёт каждому
узлу новый идентификатор и сохраняет `label`, `hidden` и `locked`. Скопированный слой вставляет своих потомков.

Live link отправляет отдельные узлы в той же форме, без `children`.

## Пример { #example }

Сокращённый фрагмент `godot/demo/maps/scripted_scene.tdm` в том виде, как его печатает `trenchdot --dump`, с полом-брашем
и взрывоопасной бочкой:

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

Пол это плита 640 на 640 единиц толщиной 8 единиц с верхом на Y 8. Его первая грань, `[0,1,2,3]`, это сторона +X,
обход против часовой стрелки при взгляде с +X.

<div class="td-mcp" markdown>

## MCP { #mcp }

Инструменты [MCP-сервера](../mcp.md) тоже читают и возвращают узлы в этой форме JSON.

</div>
