<details>
<summary>XSD contract: <code>DimensionType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="DimensionType">
		<xs:annotation>
			<xs:documentation>DimensionType describes the structure of an ordinary dimension, which is defined as a statistical concept used (most probably together with other statistical concepts) to identify a statistical series, such as a time series, e.g. a statistical concept indicating certain economic activity or a geographical reference area. The dimension takes its semantic, and in some cases it representation, from its concept identity. A dimension can be coded by referencing a code list from its coded local representation. It can also specify its text format, which is used as the representation of the dimension if a coded representation is not defined. Neither the coded or uncoded representation are necessary, since the dimension may take these from the referenced concept.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:restriction base="BaseDimensionType">
				<xs:sequence>
					<xs:element ref="common:Annotations" minOccurs="0"/>
					<xs:element ref="common:Link" minOccurs="0" maxOccurs="unbounded"/>
					<xs:element name="ConceptIdentity" type="common:ConceptReferenceType"/>
					<xs:element name="LocalRepresentation" type="SimpleDataStructureRepresentationType" minOccurs="0"/>
					<xs:element name="ConceptRole" type="common:ConceptReferenceType" minOccurs="0" maxOccurs="unbounded"/>
				</xs:sequence>
				<xs:attribute name="urn" type="common:DimensionUrnType" use="optional"/>
			</xs:restriction>
		</xs:complexContent>
	</xs:complexType>
```

</details>
