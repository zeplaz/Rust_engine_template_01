from lark import Lark, Transformer, v_args


rust_grammar = """
   start: item+ | use_stmt+
    
    use_stmt: "use" path ("as" CNAME)? -> use_import
    path: CNAME ("::" CNAME)*

    item: enum_item | ANY+

    enum_item: "pub enum" CNAME "{" enum_body "}"
    enum_body: variant ("," variant)* ","

    variant: CNAME "(" CNAME ")" -> variant_with_data
           | CNAME "{" struct_body "}" -> variant_with_struct_data
           | CNAME -> simple_variant

    struct_body: field ("," field)*
    field: CNAME ":" CNAME

    ANY: /./

    %import common.CNAME
    %import common.WS
    %ignore WS
"""

class RustTransformer(Transformer):
    @v_args(inline=True)
    def enum_item(self, name, body):
        return ("enum", name, body)

    @v_args(inline=True)
    def variant_with_data(self, name, data_type):
        return (name, data_type)

    @v_args(inline=True)
    def variant_with_struct_data(self, name, fields):
        return (name, fields)

    def simple_variant(self, name):
        return (name[0], None)

    def enum_body(self, items):
        return items
    
    def path(self, parts):
        return "::".join(parts)
        
    def use_import(self, items):
    # Handle the items as you see fit
        return items

    @v_args(inline=True)
    def field(self, name, field_type):
        return (name, field_type)

    def struct_body(self, fields):
        return fields

@v_args(inline=True)
def rust_to_fbs(rust_code):
    parser = Lark(rust_grammar, parser="lalr", transformer=RustTransformer())
    parsed_data = parser.parse(rust_code)

    fbs_output = ""

    for item_type, name, body in parsed_data.children:
        if item_type == "enum":
            fbs_output += f"enum {name} : byte {{\n"
            for variant_data in body:
                variant_name = variant_data[0]
                data_type = variant_data[1]
                if not isinstance(data_type, list):
                    fbs_output += f"    {variant_name},\n"
            fbs_output += "}\n\n"

            # Handle associated data
            for variant_data in body:
                variant_name = variant_data[0]
                data_type = variant_data[1]
                if data_type and not isinstance(data_type, list):
                    fbs_output += f"table {variant_name} {{\n"
                    fbs_output += f"    type: {data_type};\n"  
                    fbs_output += "}\n\n"
                elif isinstance(data_type, list):
                    fbs_output += f"table {variant_name} {{\n"
                    for field_name, field_type in data_type:
                        fbs_output += f"    {field_name}: {field_type};\n"
                    fbs_output += "}\n\n"

    return fbs_output

if __name__ == "__main__":
    with open(r"src/entities/types_of/s_flagz.rs", 'r') as f:
        rust_code = f.read()

    fbs_code = rust_to_fbs(rust_code)

    with open("output.fbs", 'w') as f:
        f.write(fbs_code)

    print("Generated output.fbs")