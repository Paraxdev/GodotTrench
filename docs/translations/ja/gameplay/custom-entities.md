---
source: 0c1b9cb979394402d977a6f647062a1558b8af7c
---

# GDScriptで作るオリジナルエンティティ { #custom-entities-in-gdscript }

どんなスクリプトでも、エンティティになれます。アウトプットごとに**シグナル**を、インプットごとに**メソッド**を宣言して、何かが起きたらシグナルをemitします。このアラームには、アウトプットが`tripped`の1つ、インプットが`trip`と`disarm`の2つあります。

```gdscript
@tool
class_name Alarm extends Node3D

signal tripped(activator: Node)

@export var armed := true

func _func_godot_apply_properties(props: Dictionary) -> void:
	armed = TrenchDotIO.to_bool(props.get("armed", armed))

func trip(activator: Node = null) -> void:
	if armed:
		tripped.emit(activator)

func disarm() -> void:
	armed = false
```

アクティベーターは、シグナルの最初の`Node`引数にしてください。そうすれば、チェーンの先でも、誰が始めたのかが分かります。

マップのキーは、`_func_godot_apply_properties`に届きます。`TrenchDotIO.to_bool`、`to_vector3`、`to_color`を通して読んでください。これらは、テキストでも、すでに変換された値でも受け付けます。そこで追加したノードは、シーンと一緒に保存されます。`_ready`で追加したノードは保存されず、シーンを読み込むたびに作り直されるので、実行中のゲームにしか要らないものに向いています。*Scene File*を持つエンティティでは、Godotは、そのシーンから来たノードを、シーンファイルが持つとおりに保存するので、それらのノードに加えた変更は失われます。追加したノードは、これまでどおり保存されます。シーン自身のノードへの変更は、`_ready`に書いてください。

マップはGodotエディターの中でビルドされ、そこでメソッドを実行するのはtoolスクリプトだけなので、スクリプトには`@tool`が必要です。

## 配置できるようにする { #making-it-placeable }

エディターとGodotは、エンティティクラスのことを、プロジェクトのFGD(エンティティ定義の一覧)から知ります。定義はそれぞれ独立したリソースで、選ぶ種類によってGodotが何をビルドするかが決まります。

| あなたのエンティティ | 定義 | 設定すること |
| --- | --- | --- |
| ポイントエンティティ。ランプやピックアップのように、一点にあるもの | `FuncGodotFGDPointClass` | *Script Class*に自分のスクリプトを設定する。代わりにシーンをスポーンするなら*Scene File*を設定する。その場合、*Script Class*はシーンのルートのスクリプトを置き換える |
| ブラシエンティティ。ドアやトリガーのように、ブラシから作られるもの | `FuncGodotFGDSolidClass` | *Script Class*に自分のスクリプトを、*Node Class*にそれが継承するボディ(`StaticBody3D`、`AnimatableBody3D`、`Area3D`)を設定する。空のままだと、Godotはスクリプトが継承するボディをビルドし、スクリプトが`Node3D`を継承しているか、継承がなければ`StaticBody3D`をビルドする。[衝突](entities/collision.md)を参照 |

上のアラームの定義は、次のとおりです。スクリプト`alarm.gd`の隣に、`res://entities/alarm.tres`として保存します。

```
[gd_resource type="Resource" script_class="FuncGodotFGDPointClass" load_steps=3 format=3]

[ext_resource type="Script" path="res://addons/func_godot/src/fgd/func_godot_fgd_point_class.gd" id="1_class"]
[ext_resource type="Script" path="res://entities/alarm.gd" id="2_script"]

[resource]
script = ExtResource("1_class")
classname = "alarm"
description = "Trips when told to, until disarmed"
script_class = ExtResource("2_script")
class_properties = Dictionary[String, Variant]({
"targetname": "",
"armed": true
})
node_class = "Node3D"
```

これを、マップ設定とゲーム設定の両方が使うFGDの*Entity Definitions*に追加してください。[プロジェクトの設定](../godot/project-setup.md#your-own-resources)を参照してください。するとゲーム設定が、これをエディターへエクスポートします。シグナルはアウトプットに、`_`で始まらないメソッドはインプットになります。それでもエンティティが普通のノードとしてビルドされるときは、[何も起きないエンティティ](../godot/entities-troubleshooting.md)を参照してください。

手早く始めるには、**リファレンス**パネルを開いて、自分のものに近いクラスを選びます。*GDScript クラス*タブで**プロジェクトに作成**を押すと、そのひな形が`res://entities/`の下に書き出されます。

![リファレンスパネルのGDScript クラスタブ。プロジェクトに作成にカーソルが重なり、ツールチップにres://entities/の下にファイルを書き出す、すでにあれば書かない、と出ている](../assets/custom-entities/reference-create-in-project.png)

*FGD.tres*タブで**プロジェクトに作成**を押すと、定義と、それが実行するスクリプトが書き出されます。どちらもクラス名の前に`my_`を付けた名前になるので、`func_button`なら`res://entities/my_func_button.tres`と`my_func_button.gd`になります。その`.tres`を自分のFGDの*Entity Definitions*に追加すると、Godotがそれをビルドします。

![リファレンスパネルのFGD.tresタブ。プロジェクトに作成にカーソルが重なり、ツールチップにres://entities/my_func_button.tresを書き出す、独自のclassnameを持つ定義と、それが実行するスクリプト、と出ている](../assets/custom-entities/reference-create-definition.png)

## コードからI/Oを呼ぶ { #calling-io-from-code }

ゲームのコードは、マップの接続と同じしくみを使えます。

```gdscript
for door in TrenchDotIO.find_targets(self, "main_door", null):
	TrenchDotIO.invoke(door, &"open", "", null)

TrenchDotIO.fire_output(self, &"tripped", activator)
```

メソッドを直接呼ぶより、`invoke`を使うことをお勧めします。マップの接続と同じように、パラメーターを引数に合わせ、組み込みのインプットも扱います。`fire_output`は、そのアウトプットのすべての接続を手動で起動します。その名前のシグナルをエンティティが持たないときに便利です。
