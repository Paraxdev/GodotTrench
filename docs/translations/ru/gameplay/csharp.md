---
source: 602da12da979c292b377fb3a10cadcc22f1babd5
---

# Сущности на C# { #c-entities }

Класс C# может быть размещаемой сущностью без определения `.tres`. Аддон читает ваши исходные файлы: сигналы становятся выходами, экспортируемые члены становятся ключами, а помеченные методы становятся входами.

Для запуска сущностей на C# нужна .NET-сборка Godot. Редактор перечисляет их в любом случае, но Godot может собрать такую сущность, только когда он запущен в .NET-сборке, проект C# собран и у класса есть `[GlobalClass]`. Иначе браш-сущность на C# собирается как обычный `Node3D` без коллизии, а точечная сущность как пустой, и панель Output в Godot пишет `Node Class ... not found` с этими тремя пунктами для проверки.

## 1. Добавьте помощник { #1-add-the-helper }

Откройте панель **Справка** (*Вид > Панели > Справка*) и нажмите **Добавить C#-помощник в проект** слева вверху.

![Панель «Справка» с наведённой кнопкой «Добавить C#-помощник в проект», подсказка гласит: записывает TrenchDot.cs, мост для сущностей на C#](../assets/csharp/reference-csharp-helper.png)

Он записывает `res://addons/func_godot/csharp/TrenchDot.cs`, где лежат атрибуты. Без него код вашей сущности не компилируется.

## 2. Объявите сущность { #2-declare-an-entity }

```csharp
[GlobalClass]
[TrenchDotEntity("npc_guard", Description = "Patrolling guard",
    Color = "#ff4040ff", Size = "-16 0 -16 16 64 16")]
public partial class NpcGuard : CharacterBody3D
{
    [Signal] public delegate void AlertedEventHandler(Node activator, int level); // output "alerted"
    [Export] public float WalkSpeed { get; set; } = 3.5f;                          // key "walk_speed"
    [TrenchDotInput] public void Alert(Node activator) { }                        // input "alert"
}
```

| Часть | Почему это важно |
| --- | --- |
| `[GlobalClass]` | Обязателен, сборка карты создаёт узел по имени его класса |
| `Size` | Коробка, которую редактор рисует для сущности, сначала минимумы, затем максимумы в единицах карты |
| `Solid = true` | Делает браш-сущность, например дверь, вместо точечной сущности |
| `[TrenchDotInput]` | Помечает входы. Если его нет ни у одного метода, входом становится каждый публичный метод `void` с именем с заглавной буквы |

Имена в карте пишутся в snake_case, а в C# в PascalCase. Чтобы читать ключи самостоятельно, реализуйте `_func_godot_apply_properties(Dictionary props)` и пользуйтесь читателями помощника вроде `TrenchDot.Float(props, "walk_speed", 3.5f)`.

> [!TIP]
> Вкладка *Класс C#* панели «Справка» записывает заготовку с сигналами, экспортами и входами любой сущности, быстрый старт для вашей собственной. В ней нет строки `[TrenchDotEntity]`, добавьте её сами.

## Где ищет аддон { #where-the-addon-looks }

Редактор и сборка карты по отдельности сканируют папки в поисках классов сущностей на C#, это задаётся в двух местах:

| Кто использует | Настройка |
| --- | --- |
| Редактор (экспорт конфигурации игры) | *Csharp Source Dirs* в ресурсе конфигурации игры |
| Сборка карты | Настройка проекта `trenchdot/csharp_entity_dirs` |

Обе по умолчанию равны `res://` и пропускают `addons`, `bin`, `obj` и скрытые папки.

> [!NOTE]
> Класс C# не может заменить сущность из библиотеки. Если FGD уже определяет её classname, побеждает FGD.

## I/O из C# { #io-from-c }

```csharp
var guide = TrenchDot.FindTargets(this, "guide")[0];
guide.Connect("finished", Callable.From(() => GD.Print("cutscene over")));

foreach (Node barrel in TrenchDot.FindTargets(this, "barrel"))
    TrenchDot.Invoke(barrel, "ignite", "", this);

TrenchDot.FireOutput(this, "alerted", player);
```

Вкладка *Использование в C#* панели «Справка» генерирует этот код для любой сущности.

<div class="td-mcp" markdown>

## MCP { #mcp }

Агент или скрипт, управляющий редактором через [MCP-сервер](../mcp.md), получает тот же сгенерированный код от инструмента `code_reference`.

</div>
