import sys
from clang.cindex import *

def FindStruct(node, name):

    if node.kind is CursorKind.STRUCT_DECL and node.spelling == name:
        return node
    for c in node.get_children():
        u = FindStruct(c, name)
        if u:
            return u
    return None

def PrintTypes(scursor):
    offset = 0
    for c in scursor.get_children():
        if c.kind is not CursorKind.FIELD_DECL:
            print c.kind
            continue
        var =  ''.join(x for x in c.spelling.title() if x is not '_')
        if var == "Cacheline1":
            assert(offset < 64)
            offset = 64
        if c.type.get_size() == 0:
            continue
        type = c.type.spelling if c.type.kind is not TypeKind.POINTER else "IntPtr"
        print offset, var, type, c.type.get_size()
        offset += c.type.get_size() 

if __name__ == "__main__":
    f = sys.argv[1]
    index = Index.create()
    tu = index.parse(sys.argv[1], ["-DRTE_NEXT_ABI"])
    cursor = FindStruct(tu.cursor, "rte_mbuf")
    print cursor.location.line
    PrintTypes(cursor)
