<details>
<summary>XSD contract: <code>CodelistExtensionType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="CodelistExtensionType">
		<xs:annotation>
			<xs:documentation>CodelistExtensionType defines the structure of a codelist to be extended by the codelist defining the extension. It provides a reference to the extended codelist and selection criteria to indicate the codes to be included in the extending codelist.</xs:documentation>
		</xs:annotation>
		<xs:sequence>
			<xs:element name="Codelist" type="common:CodelistReferenceType"/>
			<xs:choice minOccurs="0">
				<xs:element name="InclusiveCodeSelection" type="CodeSelectionType">
					<xs:annotation>
						<xs:documentation>Selection of codes from the referenced (extended) codelist to be included in the extending codelist.</xs:documentation>
					</xs:annotation>
				</xs:element>
				<xs:element name="ExclusiveCodeSelection" type="CodeSelectionType">
					<xs:annotation>
						<xs:documentation>Selection of codes from the referenced (extended) codelist to be excluded in the extending codelist.</xs:documentation>
					</xs:annotation>
				</xs:element>
			</xs:choice>
		</xs:sequence>
		<xs:attribute name="prefix" type="xs:string" use="optional">
			<xs:annotation>
				<xs:documentation>A reference to a codelist may contain a prefix. If a prefix is provided, this prefix will be applied to all the codes in the codelist before they are imported into the extended codelist.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
	</xs:complexType>
```

</details>
