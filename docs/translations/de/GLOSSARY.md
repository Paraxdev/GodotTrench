---
source: 62dc81f5978f85583c7c7cd6083e20459a306923
---

# Glossar { #glossary }

## 3D-Cursor { #3d-cursor }
Der Punkt in der Welt unter deiner Maus, in der Ansicht, über der du zuletzt warst. Alles, was du am Cursor platzierst, landet dort, etwa eine Entität über die Schaltfläche Platzieren im Panel Entitäten oder ein neues Terrain.

## Aktivator { #activator data-forms="Aktivators" }
Der Spieler oder Körper, der eine Kette von Entity-I/O ausgelöst hat, meistens wer den Knopf gedrückt hat oder in den Trigger gelaufen ist. Spätere Schritte der Kette können !activator als Ziel angeben und wirken dann genau auf diesen einen.

## Albedo { #albedo }
Die Grundfarbe eines Materials, die reine Textur, bevor Beleuchtung, Normal-Maps oder Leuchten dazukommen. Godot nennt die Farbtextur eines Materials seine Albedo-Textur.

## Alpha Scissor { #alpha-scissor }
Ein Transparenzmodus von Godot, der eine Textur an einem Alpha-Schwellwert hart abschneidet, sodass jedes Pixel entweder ganz sichtbar oder ganz unsichtbar ist. Er passt zu Gittern, Blättern und Decals mit harten Kanten, und anders als Alpha-Blending wirft er noch Schatten.

## Autoload { #autoload data-forms="Autoloads" }
Ein Skript oder eine Szene, die Godot einmal beim Start lädt und für das ganze Spiel am Leben hält, eingerichtet unter Project Settings > Globals > Autoload. Spielweiter Code lebt darin, zum Beispiel ein Punktezähler oder der Hot-Reload-Listener.

## Gebackene Beleuchtung { #baked-lighting data-forms="gebackenen Beleuchtung|gebackener Beleuchtung" }
Licht, das einmal im Voraus berechnet und gespeichert wird, statt in jedem Frame neu ausgerechnet zu werden. Godot > Beleuchtung backen backt Sonne, Himmel, Lampen und leuchtende Materialien in eine Lightmap, und Godot zeichnet sie über ein LightmapGI.

## Bloom { #bloom }
Das weiche Leuchten, das helle Lampen und leuchtende Materialien in die Pixel um sich herum streuen. Eingeschaltet wird es mit dem worldspawn-Key glow_intensity.

## Boolean { #boolean data-forms="Booleans" }
Ein Wert, der entweder wahr oder falsch ist, etwa ob der Strom an ist. Die Entitäten logic_branch, logic_gate und logic_flipflop speichern und verknüpfen Booleans.

## Brush { #brush data-forms="Brushes" }
Ein konvexer Körper, etwa ein Quader oder ein Keil, der Grundbaustein eines Levels. Du zeichnest Brushes in den Ansichten und kombinierst sie mit CSG, und Godot macht aus jedem ein Mesh mit passender Kollision.

## Brush-Entität { #brush-entity data-forms="Brush-Entitäten" }
Eine Entität, die ihre Form von Brushes bekommt, die du gebaut hast, etwa eine Tür, ein Knopf oder ein Trigger. Du zeichnest zuerst die Brushes und machst dann die Entität daraus, anders als bei einer Punkt-Entität, die an einer einzelnen Stelle sitzt.

## konvex { #convex data-forms="konvexe|konvexen|konvexer|konvexes" }
Überall nach außen gewölbt, ohne Dellen oder Mulden, wie ein Quader, ein Keil oder ein Zylinder. Jeder Brush muss konvex sein, deshalb lehnt der Editor Vertex-Verschiebungen ab, die ihn eindellen würden. Für konkave Formen wandelst du den Brush in ein Mesh um.

## Cordon { #cordon data-forms="Cordons" }
Eine Box, die die Karte beim Arbeiten auf einen Bereich begrenzt. Ist der Cordon aktiv, sind Objekte außerhalb der Box ausgeblendet und fehlen in einem Export mit Cordon.

## CSG { #csg }
Constructive Solid Geometry, also Formen bauen, indem man Körper kombiniert, etwa einen Brush von einem anderen abziehen, um einen Türdurchgang auszuschneiden. Die CSG-Befehle des Editors sind Subtrahieren, Aushöhlen, Konvex vereinen und Schnittmenge.

## Decal { #decal data-forms="Decals" }
Eine Textur, die über eine Oberfläche gelegt wird, wie ein Fleck, ein Riss oder ein Poster. In TrenchDot ist ein Decal ein dünnes Mesh, das ohne Flackern auf der Fläche sitzt, platziert, indem du beim Ablegen eines Materials auf einer Fläche Alt gedrückt hältst. Ein projiziertes Decal ist dagegen eine Entität, die ein Material auf alles in ihrer Box projiziert.

## Enttiling { #detile }
Eine Einstellung einer Terrainebene von 0 bis 1, die jede Wiederholung der Textur zufällig dreht und verschiebt, damit ein großes Feld kein offensichtliches Raster zeigt. Sie kostet zusätzliche Texturzugriffe, lass sie also auf 0, wo die Wiederholung nicht auffällt.

## Displacement { #displacement data-forms="Displacements" }
Eine vierseitige Brush-Fläche, die in ein Raster aus Vertices zerlegt ist, die sich einzeln heben und senken lassen, wie in Hammer. Sie liefert unebenen Boden in einem Level aus Brushes. Große Außenbereiche baust du besser als Terrain.

## Entitätsdefinition { #entity-definition data-forms="Entitätsdefinitionen" }
Die Beschreibung einer Entitätsklasse in einer FGD: ihr Classname, ihre Keys, Inputs und Outputs und was Godot dafür baut, etwa die Node-Klasse und das Skript. In Godot ist es eine Ressource vom Typ FuncGodotFGDPointClass oder FuncGodotFGDSolidClass, aufgelistet unter Entity Definitions einer FGD-Datei.

## Abfall { #falloff data-forms="Abfalls" }
Wie ein Malpinsel, etwa der des Werkzeugs Mischen, von der Mitte zum Rand ausblendet. Ein weicher Abfall gibt weiche Ränder, ein konstanter malt die ganze Scheibe mit gleicher Stärke.

## FGD { #fgd }
Die Definition der Entitätsklassen eines Spiels mit ihren Namen, Keys, Inputs und Outputs. Der Name kommt von Hammers Forge-Game-Data-Dateien. In Godot ist es eine Ressource vom Typ FuncGodotFGDFile, und die Spielkonfiguration reicht sie an den Editor weiter.

## Fixup { #fixup data-forms="Fixups" }
Ein Namenspräfix, das an einer Prefab-Instanz gesetzt wird. Mit dem Fixup p1 heißt eine Entität namens door darin p1-door, sodass mehrere Kopien eines Prefabs ihre Verdrahtung getrennt halten.

## func_detail { #func_detail }
Eine FuncGodot-Brush-Entität für statische Geometrie mit Kollision, die als eigener Node getrennt von den Welt-Brushes gebaut wird. Sie fügt keinen Occluder hinzu und verdeckt deshalb nie die Objekte dahinter vor dem Rendering.

## func_illusionary { #func_illusionary }
Eine FuncGodot-Brush-Entität, die sichtbar ist, aber keine Kollision hat, sodass Spieler hindurchgehen. Das Tutorial zur Seeinsel nutzt eine für das Wasser.

## FuncGodot { #funcgodot }
Das Godot-Plugin, das Level aus Karten-Dateien im Quake-Stil baut. Das TrenchDot-Addon ist ein Fork davon, deshalb heißt sein Node FuncGodotMap und seine Ressourcen beginnen mit FuncGodot.

## Spielkonfiguration { #game-config data-forms="Spielkonfigurationen" }
Die Datei trenchdot_game.json im Wurzelordner des Godot-Projekts, vom Addon geschrieben. Sie sagt dem Editor, welche Entitäten, Texturordner und Einheitenskala dein Projekt nutzt, und der Editor lädt sie neu, sobald das Addon sie erneut schreibt.

## Gameplay-Entitätenpaket { #gameplay-entities-pack data-forms="Gameplay-Entitätenpakets|Gameplay-Entitätenpakete" }
Die Türen, Knöpfe, Aufzüge, Trigger, Logik, Spawner, Props und Effekte der Entitätsreferenz. Sie sind kein Teil des Addons selbst. Beim Installieren des Pakets werden ihre Definitionen und Skripte nach res://trenchdot/entities kopiert, wo sie zu deinem Projekt gehören und du sie frei ändern kannst.

## glTF { #gltf }
Ein offenes Dateiformat für 3D-Modelle und Szenen, das Blender, Godot und die meisten 3D-Tools lesen. Eine .glb-Datei ist die binäre Form, die Geometrie und Texturen in einer Datei enthält.

## Godot-Gruppe { #godot-group data-forms="Godot-Gruppen" }
Ein benanntes Tag an Nodes in Godot, gesetzt unter Node > Groups oder per Code, mit dem Code und Entity-I/O alle auf einmal finden können. Ein Ziel wie @enemies erreicht jeden Node in der Gruppe enemies.

## Hammer { #hammer }
Valves Level-Editor für Source-Engine-Spiele wie Half-Life 2. TrenchDot übernimmt sein Modell aus Inputs und Outputs der Entitäten und kann seine .vmf-Karten importieren.

## Hot Reload { #hot-reload }
Die Karten eines bereits laufenden Spiels bei jedem Speichern in TrenchDot neu bauen, sodass du es nicht neu starten musst, um eine Änderung zu sehen. Dafür braucht das Spiel das Autoload trenchdot_hot_reload.gd.

## Hotspot { #hotspot data-forms="Hotspots" }
Ein Rechteck, das auf einem Trim Sheet markiert und in der Datei .hotspots.json der Textur gespeichert wird. Hotspot-Anpassung rastet die gewählten Flächen auf das Rechteck ein, das am besten zu ihnen passt, sodass ein Trim Sheet einen Raum schnell texturieren kann.

## Lightmap { #light-map data-forms="Lightmaps" }
Eine Textur, die das Licht enthält, das jede Stelle der statischen Geometrie erreicht, Reflexionen eingeschlossen. Jede Fläche bekommt darin ihren eigenen Bereich.

## Lichtsonde { #light-probe data-forms="Lichtsonden" }
Ein Punkt im offenen Raum, der das Licht speichert, das aus allen Richtungen ankommt. Bewegte Objekte beziehen ihr Licht von den Sonden um sie herum, sodass sie zur gebackenen Beleuchtung der statischen Geometrie passen.

## LightmapGI { #lightmapgi }
Der Godot-Node, der eine Lightmap und ihre Lichtsonden zeichnet. Eine Karte aus einer gebackenen .tdm bekommt einen namens baked_lighting.

## Beleuchtete Vorschau { #lit-preview }
Ein Shading-Modus der 3D-Ansicht des Editors, umgeschaltet mit F3, der die Lichter, Sonnenschatten, den Himmel und den Nebel der Karte zeigt. Gut, um die Beleuchtung zu beurteilen, bevor du in Godot baust, aber er zeichnet nur die 64 stärksten Punkt- und Spotlichter.

## Live Link { #live-link }
Die Verbindung zwischen TrenchDot und dem Godot-Editor, ein lokaler Server, den das Addon auf Port 7842 betreibt. Neu bauen beim Speichern, der Live-Modus und der Godot-Status in der Werkzeugleiste laufen alle darüber.

## Live-Modus { #live-mode }
Schiebt deine Änderungen schon beim Machen, vor dem Speichern, in die Szene, die im Godot-Editor offen ist, sodass du Beleuchtung und Maßstab in Godots eigenem Renderer beurteilen kannst. Schalte ihn mit der Link-Schaltfläche in der Werkzeugleiste oder über Godot > Live-Modus ein.

## Logikgatter { #logic-gate }
Eine Regel, die zwei Wahr-oder-falsch-Eingaben in eine Antwort verwandelt, etwa and (beide sind wahr) oder xor (genau eine ist es). Die Entität logic_gate bietet die gängigen an.

## Hauptszene { #main-scene data-forms="Hauptszenen" }
Die Szene, die Godot abspielt, wenn du das Projekt startest, eingestellt unter Project > Project Settings > Application > Run. Projekt ausführen in TrenchDot und die Play-Schaltfläche in Godot starten beide diese Szene.

## Karteneinstellungen { #map-settings }
Eine FuncGodotMapSettings-Ressource, die festlegt, woher Texturen, Materialien und Entitätsdefinitionen kommen und wie viele Karteneinheiten einen Meter ergeben. Eine FuncGodotMap kann eigene haben, sonst gilt der Projektstandard.

## MCP { #mcp }
Model Context Protocol, ein Standardweg für KI-Agenten und Skripte, die Werkzeuge eines Programms aufzurufen. Der Editor kann einen MCP-Server betreiben, damit ein Agent Karten untersuchen und bearbeiten kann. Du brauchst ihn nicht, um Level zu bauen, und diese Doku blendet die MCP-Teile aus, bis du den MCP-Schalter in der Kopfzeile einschaltest.

## MultiMeshes { #multimeshes }
Godots Art, viele Kopien eines Meshes in einem einzigen Durchgang zu zeichnen. Verteilte Bäume, Felsen und Gras werden als MultiMeshes gebaut, sodass Tausende davon günstig zu rendern bleiben.

## Navigationsmesh { #navigation-mesh data-forms="Navigationsmeshes" }
Ein vereinfachter Umriss der Böden, auf denen KI-Agenten laufen können und mit dem Godot Wege findet. TrenchDotNav backt eines aus der Kollision einer gebauten Karte.

## Normal-Map { #normal-map data-forms="Normal-Maps" }
Eine Textur, die speichert, wohin jedes Pixel einer Oberfläche zeigt, sodass ebene Flächen das Licht einfangen, als hätten sie Beulen und Rillen. Sie bringt feine Details ohne zusätzliche Geometrie.

## Operand { #operand data-forms="Operanden" }
Die Zahl, mit der eine Rechnung außer der, die sie erhält, arbeitet. In einem math_calc, das 5 addiert, ist der Operand 5.

## ORM { #orm }
Eine Textur, die drei Graustufenkarten in ihre Farbkanäle packt, Ambient Occlusion in Rot, Roughness in Grün und Metallic in Blau. Godot-Materialien und TrenchDot-Terrainebenen lesen sie als eine Datei.

## Punkt-Entität { #point-entity data-forms="Punkt-Entitäten" }
Eine Entität, die an einer einzelnen Stelle sitzt, statt ihre Form von Brushes zu bekommen, etwa ein Licht, ein Geräusch oder ein Spawnpunkt. Du platzierst eine, indem du sie aus dem Panel Entitäten in eine Ansicht ziehst.

## Prefab { #prefab data-forms="Prefabs" }
Ein wiederverwendbarer Teil einer Karte, der als eigene .tdm-Datei gespeichert und in anderen Karten als Instanzen platziert wird. Wenn du die Prefab-Datei bearbeitest, aktualisiert das jede Instanz davon.

## res:// { #res }
Godots Schreibweise für einen Pfad im Projektordner, egal wie der Ordner auf der Platte heißt. res://maps/room.tdm ist room.tdm im Ordner maps deines Godot-Projekts. Deshalb müssen Karten und Modelle im Projekt gespeichert sein, damit das Addon sie findet.

## Eingeschränkter Modus { #restricted-mode data-forms="Eingeschränkten Modus" }
So öffnet TrenchDot ein Godot-Projekt, das Code enthält und dem du noch nicht vertraust. Du kannst seine Karten ansehen und bearbeiten, aber der Editor startet Godot nicht darauf, baut nicht in Godot und sendet keine Live-Änderungen, denn Godot führt die Skripte eines Projekts aus, wenn es es öffnet, abspielt oder baut. Klick in der Statusleiste auf Eingeschränkter Modus, um dem Projekt zu vertrauen.

## Scatter-Set { #scatter-set data-forms="Scatter-Sets" }
Eine Gruppe von Modellen, die zusammen mit dem Scatter-Werkzeug gemalt wird, etwa ein Wald oder eine Graswiese, mit eigener Ebene, Dichte und eigenen Zieloberflächen. Godot baut jedes Set als MultiMeshes.

## Bildschirmraum-Reflexionen { #screen-space-reflections }
Ein Godot-Effekt, der spiegelt, was schon auf dem Bildschirm ist, in glänzenden Oberflächen, sodass nasse Straßen die Lampen spiegeln. Schalte ihn mit dem worldspawn-Key ssr auf 1 ein.

## Signal { #signal data-forms="Signale|Signalen|Signals" }
Godots Name für ein Ereignis, das ein Node meldet, etwa wenn ein Knopf gedrückt wird. Jeder Output einer Entität ist ein Signal, und einen Output zu verdrahten verbindet dieses Signal mit einer Input-Methode am Ziel.

## targetname { #targetname }
Der Name, den du einer Entität gibst, damit andere Entitäten sie erreichen, etwa main_door. Outputs, Prefab-Fixups und Keys wie fixture finden Entitäten über ihn.

## Texel { #texel data-forms="Texeln" }
Ein Pixel einer Textur, wie es auf einer Oberfläche landet. Standardmäßig deckt ein Texel eine Karteneinheit ab, und die Texeldichte, wie viele Texel auf eine Fläche passen, bestimmt, wie scharf eine Textur aussieht. Auch eine Lightmap hat Texel, und ihre Texelgröße sagt, wie viele Karteneinheiten eines abdeckt.

## Texturgröße { #texture-size data-forms="Texturgrößen" }
Die Weltgröße, die eine Wiederholung einer Textur abdeckt, eingestellt als metadata/texture_size an einem Godot-Material. Ohne sie deckt ein Pixel eine Karteneinheit ab, was ein hochauflösendes Foto über eine riesige Fläche zieht.

## Werkzeugtexturen { #tool-textures data-forms="Werkzeugtextur" }
Spezielle Texturen, die dem Build sagen, wofür eine Brush-Fläche da ist, statt sie zu zeichnen. Clip-Flächen kollidieren nur, Skip-Flächen bauen nichts, und Origin-Flächen legen fest, wo der Ursprung einer Brush-Entität liegt. Jede Textur in den Ordnern special/, tools/ oder td/ zeichnet in Godot nichts, genauso nodraw, hint, trigger und die anderen Werkzeugnamen in jedem Ordner.

## TrenchBroom { #trenchbroom }
Ein kostenloser Level-Editor für Spiele im Quake-Stil. TrenchDot holt sich viel Inspiration davon und kann seine .map-Dateien importieren.

## Trigger-Volumen { #trigger-volume }
Eine unsichtbare Brush-Entität, die Outputs auslöst, wenn ein Körper sie betritt oder verlässt, zum Beispiel um eine Zwischensequenz zu starten, wenn der Spieler einen Raum betritt. Zeichne eines mit dem Werkzeug Volumen (Shift+E).

## Trim Sheet { #trim-sheet data-forms="Trim Sheets" }
Eine Textur, die Streifen mit Details wie Kanten, Rohren und Paneelen in ein Bild packt. Flächen werden mit Hotspots an ihre Teile angepasst, sodass wenige Trim Sheets ein ganzes Gebäude texturieren können.

## Triplanar { #triplanar }
Eine Art, eine Textur aus drei Richtungen auf eine Oberfläche zu projizieren, von oben und von zwei Seiten, überblendet nach dem Winkel der Oberfläche. Terrainebenen nutzen es, damit Klippen Fels zeigen statt einer verzerrten Schliere.

## UV-Sperre { #uv-lock }
Eine Einstellung, standardmäßig an und umgeschaltet mit Ctrl+Shift+U, die Texturen an einem Brush haften lässt, während du ihn verschiebst. Ist sie aus, bleibt die Textur in der Welt stehen und gleitet über die verschobenen Flächen.

## UV { #uv }
Die Koordinaten, die sagen, welcher Teil einer Textur auf jeden Punkt einer Fläche fällt, U quer über die Textur und V nach unten. Brush-Flächen bekommen sie meist aus einer Projektion, während Meshes und manche Flächen für jede Ecke eigene speichern.

## worldspawn { #worldspawn }
Die Wurzel-Entität der Karte, die die Welt-Brushes enthält. Ihre Keys halten Einstellungen für die ganze Karte, etwa Himmel, Nebel, Sonne und Streaming, und der Inspektor zeigt sie, wenn nichts ausgewählt ist.

## Yaw { #yaw }
Drehung um die senkrechte Achse, die Richtung, in die etwas von oben gesehen schaut. Der Yaw eines Teleport-Ziels legt fest, in welche Richtung Körper nach der Ankunft schauen.

## Z-Fighting { #z-fighting }
Flackernde Streifen dort, wo zwei Oberflächen auf oder sehr nahe an derselben Ebene liegen. Die GPU kann nicht erkennen, welche vorne ist, also zeigt jedes Pixel mal die eine, mal die andere, und das Muster wandert mit der Kamera. Decals vermeiden es, dünne Brushes auf einer Wand nicht.
