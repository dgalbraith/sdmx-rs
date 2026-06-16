<details>
<summary>XSD contract: <code>ValueItemType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="ValueItemType">
		<xs:annotation>
			<xs:documentation>ValueItemType defines the structure of a value item. A value must be provided, and a longer name and description can be provided to provide additional meaning to the value (similar to a code in a code list).</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:extension base="common:AnnotableType">
				<xs:sequence>
					<xs:element ref="common:Name" minOccurs="0" maxOccurs="unbounded"/>
					<xs:element ref="common:Description" minOccurs="0" maxOccurs="unbounded"/>
				</xs:sequence>
				<xs:attribute name="id" type="xs:string" use="required"/>
			</xs:extension>
		</xs:complexContent>
	</xs:complexType>
```

</details>
