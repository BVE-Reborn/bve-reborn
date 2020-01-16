# Test Meshes

**This is not currently true, I currently can't normalize meshes, so results
aren't currently properly checked**

In this folder are test meshes for testing out the mesh loader. There are two files
per test, which are then included in the testing binary.

### Source file

The source file is noted by a `.obj` extension. This is a shape generated
by blender and exported without materials and without triangularization.

### Result file

The result file is noted by a `.res.obj` extension.
The exact same mesh is taken into blender, with the edge split (0 deg) and
triangularization (`fixed` method) modifiers applied, in that order. This will result in an
obj with the same properties as the parser loads.
