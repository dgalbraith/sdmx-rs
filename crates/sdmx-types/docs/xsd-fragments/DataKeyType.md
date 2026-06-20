<details>
<summary>XSD contract: <code>DataKeyType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="DataKeyType">
		<xs:annotation>
			<xs:documentation>DataKeyType is a region which defines a distinct full or partial data key. The key consists of a set of values, each referencing a dimension and providing a single value for that dimension. The purpose of the key is to define a subset of a data set (i.e. the observed value and data attribute) which have the dimension values provided in this definition. Any dimension not stated explicitly in this key is assumed to be wild carded, thus allowing for the definition of partial data keys.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:restriction base="RegionType">
				<xs:sequence>
					<xs:element ref="common:Annotations" minOccurs="0"/>
					<xs:element name="KeyValue" type="DataKeyValueType" minOccurs="0" maxOccurs="unbounded"/>
					<xs:element name="Component" type="DataComponentValueSetType" minOccurs="0" maxOccurs="unbounded"/>
				</xs:sequence>
				<xs:attribute name="include" type="xs:boolean" use="optional" fixed="true">
					<xs:annotation>
						<xs:documentation>The include attribute has a fixed value of true for a distinct key, since such a key is always assumed to identify existing data or metadata.</xs:documentation>
					</xs:annotation>
				</xs:attribute>
			</xs:restriction>
		</xs:complexContent>
	</xs:complexType>
```

</details>
